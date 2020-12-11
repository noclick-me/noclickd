mod cli;
mod config;
mod service;
mod state;
mod url_info;
mod urlize;

use crate::config::Config;
use crate::state::SharedState;

use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = cli::create_parser().get_matches();
    println!("Binding to: {}...", matches.value_of("bind").unwrap());

    let state = web::Data::new(SharedState::new(Config::default()));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(service::url::mount(web::scope("/url")))
            .service(service::view::mount(web::scope("/")))
    })
    .bind(matches.value_of("bind").unwrap())?
    .run()
    .await
}
