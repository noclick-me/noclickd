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
async fn url_get(
    path: web::Path<(String,)>,
    db_pool: web::Data<sqlx::SqlitePool>,
) -> impl Responder {
    let (id,) = path.into_inner();
    dbg!(&id);

    let mut db_conn = db_pool.acquire().await.unwrap();
    let row = sqlx::query(r"SELECT source_url, noclick_url FROM urls WHERE id = $1")
        .bind(&id)
        .fetch_one(&mut db_conn)
        .await
        .unwrap();

    use sqlx::Row;
    let source_url: &str = row.try_get("source_url").unwrap();
    let response = UrlCreateRs {
        id: &id,
        source_url: &source_url,
        noclick_url: row.try_get("noclick_url").unwrap(),
    };

    use actix_web::HttpResponse;
    HttpResponse::Ok().json(response)
}

#[post("")]
async fn url_post(
    rq: web::Json<UrlCreateRq>,
    id_state: web::Data<IdState>,
    db_pool: web::Data<sqlx::SqlitePool>,
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

    use crate::config::config;
    let source_url = info.url.as_ref().unwrap().clone(); // URL should always exist here
    let expanded_url = info.urlize(config().link.max_length).unwrap();

    let mut db_conn = db_pool.acquire().await.unwrap();

    use actix_web::HttpResponse;
    while let Some(id) = idgen.next() {
        let mut noclick_url = format!("{}/{}/{}", config().link.base_url, id, expanded_url);
        noclick_url.truncate(config().link.max_length);

        let result = sqlx::query(
            r"INSERT OR ABORT INTO urls (id, source_url, noclick_url) VALUES ($1, $2, $3)",
        )
        .bind(&id)
        .bind(&source_url)
        .bind(&noclick_url)
        .execute(&mut db_conn)
        .await;
        match result {
            Err(error) => {
                // TODO: check error
                dbg!(&error);
                continue;
            }
            Ok(_) => {
                id_state.id_length.set(idgen.current_length);
                let response = UrlCreateRs {
                    id: &id,
                    source_url: &source_url,
                    noclick_url,
                };
                return HttpResponse::Ok().json(response);
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
