use crate::state::SharedState;

use actix_web::{get, http, web, HttpResponse, Responder, Scope};

pub fn mount(scope: Scope) -> Scope {
    scope.service(view)
}

#[get("{id}")]
async fn view(
    web::Path((id,)): web::Path<(String,)>,
    state: web::Data<SharedState>,
) -> impl Responder {
    dbg!(&id);
    let read_db = state.db.read().unwrap();
    let entry = read_db.get(&id).unwrap();

    HttpResponse::Found()
        .header(http::header::LOCATION, entry.source_url.clone())
        .finish()
        .into_body()
}
