mod cli;
mod config;
mod service;
mod state;
mod url_info;
mod urlize;

use crate::config::Config;
use crate::state::SharedState;

use actix_web::{web, App, HttpServer};
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
    println!("Binding to: {}...", matches.value_of("bind").unwrap());

    let state = web::Data::new(SharedState::new(Config::default()));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(service::url::mount(web::scope("/url")))
            .service(service::view::mount(web::scope("/")))
    })
    .bind_rustls(matches.value_of("bind").unwrap(), tls_config())?
    .run()
    .await
}
