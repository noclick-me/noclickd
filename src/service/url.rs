use crate::config::Config;
use crate::state::{Entry, SharedState};

use actix_web::{get, post, web, Responder, Scope};
use serde::{Deserialize, Serialize};

pub fn mount(scope: Scope) -> Scope {
    let id_state = IdState::new(2);
    // create thread-local id_state so we don't need to care about sync/sharing
    scope.data(id_state).service(url_get).service(url_post)
}

#[derive(Debug)]
struct IdState {
    id_length: std::cell::Cell<usize>,
}

impl IdState {
    fn new(start_length: usize) -> Self {
        IdState {
            id_length: std::cell::Cell::new(start_length),
        }
    }
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

// TODO: This should be an actor, because we should count retries for all threads globally,
// not just the current one, otherwise when the server is under load we might have many more
// retries than we want until we increment the length.
#[derive(Debug)]
struct IdGenerator {
    current_attempts: usize,
    pub current_length: usize,
    pub max_retries: usize,
    pub increment_after: usize,
}

impl IdGenerator {
    fn new(current_length: usize) -> Self {
        IdGenerator {
            current_length,
            current_attempts: 0,
            max_retries: 10,
            increment_after: 3,
        }
    }

    fn next(&mut self) -> Option<String> {
        print!(
            "IdGenerator::next(), current_attempts={:?}",
            self.current_attempts,
        );
        self.current_attempts += 1;
        if self.current_attempts > self.max_retries {
            print!("IdGenerator::next() max attempts reached");
            return None;
        }
        if (self.current_attempts % self.increment_after) == 0 {
            self.current_length += 1;
            print!("IdGenerator::next() new length={:?}", self.current_length);
        }
        let len = self.current_length;
        let id = nanoid::nanoid!(len);
        dbg!(&id);
        Some(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn id_generator() {
        let mut gen = IdGenerator::new(1);
        assert_eq!(gen.current_length, 1);
        assert!(gen.increment_after <= gen.max_retries);

        let mut current_length = 1;
        let mut total_retries = 0;
        let mut inc_after = 0;
        while total_retries < gen.max_retries {
            total_retries += 1;
            inc_after += 1;
            let id = gen.next();
            if total_retries > gen.max_retries {
                assert!(id.is_none());
            }
            assert!(id.is_some());
            if inc_after == gen.increment_after {
                inc_after = 0;
                current_length += 1;
            }
            assert_eq!(id.unwrap().len(), current_length);
        }
    }
}

#[get("{id}")]
async fn url_get(path: web::Path<(String,)>, state: web::Data<SharedState>) -> impl Responder {
    let (id,) = path.into_inner();
    dbg!(&id);
    let read_db = state.db.read().unwrap();
    let entry = read_db.get(&id).unwrap();

    use crate::config::config;
    use actix_web::HttpResponse;
    HttpResponse::Ok().json(UrlCreateRs::from_entry(&entry, &config()))
}

#[post("")]
async fn url_post(
    rq: web::Json<UrlCreateRq>,
    id_state: web::Data<IdState>,
    state: web::Data<SharedState>,
) -> impl Responder {
    use crate::url_info::ResourceInfo;

    let info = ResourceInfo::fetch(&rq.url).await.unwrap();
    dbg!(&info);

    // We use Relaxed ordering everywhere because we don't actually need very strong guarantees of
    // this counter, is not a big deal if in some thread the update is a bit delayed or if it gets
    // incremented twice, is just a small inconvenience, but the value of this length is all a big
    // heuristic.
    let mut idgen = IdGenerator::new(id_state.id_length.get());
    dbg!(&idgen);

    let mut write_db = state.db.write().unwrap();
    use actix_web::HttpResponse;
    while let Some(id) = idgen.next() {
        use std::collections::hash_map;
        match write_db.entry(id.to_string()) {
            hash_map::Entry::Occupied(_) => continue, // retry with next ID
            hash_map::Entry::Vacant(e) => {
                use crate::config::config;
                let entry = e.insert(Entry {
                    id,
                    source_url: info.url.as_ref().unwrap().clone(), // URL should always exist here
                    noclick_url: info.urlize(config().link.max_length).unwrap(),
                });
                id_state.id_length.set(idgen.current_length);
                return HttpResponse::Ok().json(UrlCreateRs::from_entry(&entry, &config()));
            }
        };
    }
    id_state.id_length.set(idgen.current_length);

    HttpResponse::ServiceUnavailable()
        .header("Retry-After", "120")
        .json(format!(
            "{{\"error\": \"Unable to get an ID for the new URL after {} attempts\"}}",
            idgen.max_retries
        ))
}
