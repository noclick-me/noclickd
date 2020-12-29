use crate::state::SharedState;

use actix_web::{get, http, web, HttpResponse, Responder, Scope};

pub fn mount(scope: Scope) -> Scope {
    scope.service(view)
}

#[get("{id}/{rest:.*}")]
async fn view(
    web::Path((id, rest)): web::Path<(String, String)>,
    state: web::Data<SharedState>,
) -> impl Responder {
    dbg!(&id);
    dbg!(&rest);
    let read_db = state.db.read().unwrap();
    let entry = read_db.get(&id).unwrap();

    HttpResponse::Found()
        .header(http::header::LOCATION, entry.source_url.clone())
        .finish()
        .into_body()
}
