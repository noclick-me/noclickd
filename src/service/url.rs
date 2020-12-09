use crate::state::SharedState;
use crate::url_info::UrlInfo;

use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;

pub fn mount(scope: Scope) -> Scope {
    scope.service(url_get).service(url_post)
}

#[derive(Deserialize, Debug)]
struct UrlCreateRq {
    url: String,
}

#[derive(Serialize, Debug)]
struct UrlCreateRs<'a> {
    id: &'a str,
    source_url: &'a str,
    noclick_url: &'a str,
}

fn get_id() -> String {
    nanoid::nanoid!(2)
}

#[get("{id}")]
async fn url_get(
    web::Path((id,)): web::Path<(String,)>,
    state: web::Data<SharedState>,
) -> impl Responder {
    dbg!(&id);
    let read_db = state.db.read().unwrap();
    let entry = read_db.get(&id).unwrap();
    return HttpResponse::Ok().json(UrlCreateRs {
        id: &entry.id,
        source_url: &entry.source_url,
        noclick_url: &entry.noclick_url,
    });
}

#[post("")]
async fn url_post(rq: web::Json<UrlCreateRq>, state: web::Data<SharedState>) -> impl Responder {
    let info = UrlInfo::fetch(&rq.url).await.unwrap();
    dbg!(&info);
    let mut id = get_id();

    let mut write_db = state.db.write().unwrap();
    // TODO: limit looping
    loop {
        match write_db.entry(id.to_string()) {
            Entry::Occupied(_) => id = get_id(),
            Entry::Vacant(e) => {
                let entry = e.insert(crate::state::Entry {
                    id: id.clone(),
                    source_url: info.url.clone(),
                    noclick_url: info.urlize().unwrap(),
                });
                return HttpResponse::Ok().json(UrlCreateRs {
                    id: &entry.id,
                    source_url: &entry.source_url,
                    noclick_url: &entry.noclick_url,
                });
            }
        };
    }
}
