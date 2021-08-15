#![feature(async_closure)]
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use chashmap::CHashMap;

#[derive(Debug, Clone, Default)]
struct AppState {
    pub crawled_pages: CHashMap<String, Vec<String>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState::default());
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
