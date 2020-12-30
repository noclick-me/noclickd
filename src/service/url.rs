use crate::config::{config, Config};
use crate::state::{Entry, SharedState};
use crate::url_info::UrlInfo;

use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use std::collections::hash_map;

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
    noclick_url: String,
}

impl<'a> UrlCreateRs<'a> {
    fn from_entry(entry: &'a Entry, config: &Config) -> Self {
        let mut url = format!(
            "{}/{}/{}",
            config.link.base_url, entry.id, entry.noclick_url
        );
        url.truncate(config.link.max_length);
        Self {
            id: &entry.id,
            source_url: &entry.source_url,
            noclick_url: url,
        }
    }
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

    HttpResponse::Ok().json(UrlCreateRs::from_entry(&entry, &config()))
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
            hash_map::Entry::Occupied(_) => id = get_id(),
            hash_map::Entry::Vacant(e) => {
                let entry = e.insert(Entry {
                    id: id.clone(),
                    source_url: info.url.clone(),
                    noclick_url: info.urlize(config().link.max_length).unwrap(),
                });
                return HttpResponse::Ok().json(UrlCreateRs::from_entry(&entry, &config()));
            }
        };
    }
}
