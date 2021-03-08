use actix_web::{get, web, Responder, Scope};

pub fn mount(scope: Scope) -> Scope {
    scope.service(view)
}

#[get("{id}/{rest:.*}")]
async fn view(
    path: web::Path<(String, String)>,
    db_pool: web::Data<sqlx::SqlitePool>,
) -> impl Responder {
    let (id, rest) = path.into_inner();
    dbg!(&id);
    dbg!(&rest);

    let mut db_conn = db_pool.acquire().await.unwrap();
    let row = sqlx::query(r"SELECT noclick_url FROM urls WHERE id = $1")
        .bind(&id)
        .fetch_one(&mut db_conn)
        .await
        .unwrap();

    use crate::config::config;
    use actix_web::{http::header::LOCATION, HttpResponse};
    use sqlx::Row;
    let noclick_url: &str = row.try_get("noclick_url").unwrap();
    let mut url = format!("{}/{}/{}", config().link.base_url, id, noclick_url);
    url.truncate(config().link.max_length);
    HttpResponse::Found().set_header(LOCATION, url).finish()
}
