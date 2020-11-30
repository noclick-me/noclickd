use actix_web::{web, App, HttpServer, Responder};

mod cli;

async fn root() -> impl Responder {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = cli::create_parser().get_matches();

    HttpServer::new(|| App::new().route("/", web::get().to(root)))
        .bind(matches.value_of("bind").unwrap())?
        .run()
        .await
}
