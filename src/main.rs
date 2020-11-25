use actix_web::{web, App, HttpServer, Responder};

async fn index() -> impl Responder {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = clap::App::new("noclickd")
          .version(env!("CARGO_PKG_VERSION"))
          .author(env!("CARGO_PKG_AUTHORS"))
          .about("noclick.me API server")
          .arg(clap::Arg::with_name("bind")
               .short("b")
               .long("bind")
               .default_value("127.0.0.1:8080")
               .value_name("IP:PORT")
               .help("Set the address/port to bind the server to")
               .takes_value(true))
          .get_matches();

    HttpServer::new(|| {
        App::new().service(
            // prefixes all resources and routes attached to it...
            web::scope("/app")
                // ...so this handles requests for `GET /app/index.html`
                .route("/index.html", web::get().to(index)),
        )
    })
    .bind(matches.value_of("bind").unwrap())?
    .run()
    .await
}
