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
    let row = sqlx::query(r"SELECT source_url FROM urls WHERE id = $1")
        .bind(&id)
        .fetch_one(&mut db_conn)
        .await
        .unwrap();

    use actix_web::{http::header::LOCATION, HttpResponse};
    use sqlx::Row;
    let source_url: &str = row.try_get("source_url").unwrap();
    HttpResponse::Found()
        .set_header(LOCATION, source_url)
        .finish()
}
