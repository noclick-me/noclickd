use crate::config::config;

use actix_web::{get, http, HttpResponse, Responder, Scope};

pub fn mount(scope: Scope) -> Scope {
    scope.service(webapp)
}

#[get("")]
async fn webapp() -> impl Responder {
    HttpResponse::Found()
        .header(http::header::LOCATION, config().webapp.redirect_to.clone())
        .finish()
        .into_body()
}
