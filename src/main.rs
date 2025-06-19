use actix_web::{App, HttpServer};
use dotenvy::dotenv;

mod routes;
mod db;
mod handlers;
mod models;
mod config;
use routes::auctions::auctions_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db = crate::db::connect().await;

    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(db.clone()))
            .configure(auctions_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}