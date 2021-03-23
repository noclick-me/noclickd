mod cli;
mod config;
mod service;
mod url_info;
mod urlize;

use rustls::ServerConfig;
fn tls_config() -> ServerConfig {
    use rustls::internal::pemfile::{certs, pkcs8_private_keys};
    use rustls::NoClientAuth;
    use std::fs::File;
    use std::io::BufReader;

    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = pkcs8_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();
    config
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    use actix_web::{guard, middleware, web, App, HttpServer};

    use actix_ratelimit::MemoryStore;
    let rate_limit_store = MemoryStore::new();

    use crate::config::config;
    use anyhow::Context;
    use sqlx::SqlitePool;
    let db_pool = SqlitePool::connect(&config().db.url)
        .await
        .with_context(|| format!("Failed to open database '{}'", &config().db.url))?;

    // We create the needed tables if they don't exist. This is UGLY, but we don't even know if
    // we'll keep using SQL (probably not). We are just prototyping.
    sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS urls (
                id          TEXT PRIMARY KEY NOT NULL,
                source_url  TEXT NOT NULL,
                noclick_url TEXT NOT NULL
            );
        "#,
    )
    .execute(&db_pool)
    .await?;

    let matches = crate::cli::create_parser().get_matches();

    println!("Binding to: {}...", matches.value_of("bind").unwrap());
    HttpServer::new(move || {
        use actix_cors::Cors;
        let mut cors = Cors::default();
        cors = cors.allowed_headers(config().cors.allowed_headers.clone());
        cors = cors.allowed_methods(config().cors.allowed_methods.clone());
        cors = cors.max_age(60 * 60 * 24); // 24 hours
        for origin in config().cors.allowed_origins.iter() {
            cors = cors.allowed_origin(origin);
        }

        let one_day = std::time::Duration::from_secs(60 * 60 * 24);

        // Configure a global rate limit by using a constant identifier to protect against using
        // all our CPU/network quota. This won't protect us from DoS attacks, for that we need also
        // some per-ip rate limit, but for now it should be enough.
        // See https://github.com/noclick-me/noclickd/issues/31.
        use actix_ratelimit::{MemoryStoreActor, RateLimiter};
        let rate_limiter_store = MemoryStoreActor::from(rate_limit_store.clone()).start();
        let rate_limiter = RateLimiter::new(rate_limiter_store.clone())
            .with_interval(one_day) // 1 day
            .with_identifier(|_| Ok(String::from("__global__")))
            .with_max_requests(config().limits.global_requests_per_day);
        // We also allow cross-origin requests to retrieve the ratelimit headers
        cors = cors.expose_headers(vec![
            "x-ratelimit-limit",
            "x-ratelimit-remaining",
            "x-ratelimit-reset",
        ]);

        App::new()
            .wrap(rate_limiter)
            .wrap(cors)
            .wrap(middleware::Compress::default())
            .data(db_pool.clone())
            .service(service::url::mount(web::scope("/url")).guard(guard::Host(&config().api.host)))
            .service(
                service::webapp::mount(web::scope(&config().webapp.redirect_from_path)).guard(
                    guard::All(guard::Host(&config().webapp.redirect_from_host)).and(
                        guard::fn_guard(|req| req.uri.path() == config().webapp.redirect_from_path),
                    ),
                ),
            )
            .service(
                service::view::mount(web::scope("/")).guard(guard::Host(&config().viewer.host)),
            )
            .default_service(web::route().to(|| actix_web::HttpResponse::NotFound()))
    })
    .bind_rustls(matches.value_of("bind").unwrap(), tls_config())?
    .run()
    .await?;

    Ok(())
}
