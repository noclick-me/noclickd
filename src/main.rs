mod cli;
mod config;
mod service;
mod state;
mod url_info;
mod urlize;

use crate::config::Config;
use crate::state::SharedState;

use actix_cors::Cors;
use actix_web::{guard, middleware, web, App, HttpServer};

use rustls::internal::pemfile::{certs, pkcs8_private_keys};
use rustls::{NoClientAuth, ServerConfig};
use std::fs::File;
use std::io::BufReader;

fn tls_config() -> ServerConfig {
    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = pkcs8_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();
    config
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = cli::create_parser().get_matches();

    let state = web::Data::new(SharedState::new(Config::default()));

    println!("Binding to: {}...", matches.value_of("bind").unwrap());
    HttpServer::new(move || {
        let state = state.clone();
        let mut cors = Cors::default();
        cors = cors.allowed_headers(state.config.cors.allowed_headers.clone());
        cors = cors.allowed_methods(state.config.cors.allowed_methods.clone());
        cors = cors.max_age(60 * 60 * 24); // 24 hours
        for origin in state.config.cors.allowed_origins.iter() {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .wrap(cors)
            .wrap(middleware::Compress::default())
            .app_data(state)
            .service(service::url::mount(web::scope("/url")))
            .service(service::view::mount(web::scope("/")))
    })
    .bind_rustls(matches.value_of("bind").unwrap(), tls_config())?
    .run()
    .await
}
