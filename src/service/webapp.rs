use actix_web::{get, Responder, Scope};

pub fn mount(scope: Scope) -> Scope {
    scope.service(webapp)
}

#[get("")]
async fn webapp() -> impl Responder {
    use crate::config::config;
    use actix_web::{http::header::LOCATION, HttpResponse};

    HttpResponse::Found()
        .set_header(LOCATION, config().webapp.redirect_to.clone())
        .finish()
}
