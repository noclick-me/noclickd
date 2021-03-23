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
        .await;

    use actix_web::http::header;
    use actix_web::HttpResponse;
    match row {
        Ok(row) => {
            use sqlx::Row;
            let source_url: &str = row.try_get("source_url").unwrap();
            HttpResponse::Found()
                .set_header(header::LOCATION, source_url)
                .finish()
        }
        Err(_) => HttpResponse::NotFound()
            .content_type("text/html; charset=utf-8")
            .body(
                r"<h1>404 NOT FOUND</h1>
                <p>Sorry, the link wasn't found. Maybe you got it wrong or it was removed from the
                DB. Please remember this service is only a preview and anything can be removed at
                any time ¯\_(ツ)_/¯.</p>
                <p>If you want to get a stable service faster, please consider <strong><a
                href='https://github.com/llucax/llucax/blob/main/sponsoring-platforms.md'>sponsoring
                it</a></strong>!</p>",
            ),
    }
}
