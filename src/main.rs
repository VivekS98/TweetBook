mod api;
mod models;
mod utils;

use actix_web::{web, App, HttpServer};
use api::{auth::auth, messages::messages, user::user};
use models::init::Tweetbook;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Tweetbook::init().await;
    let db_data = web::Data::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .configure(auth) // Auth related routes
            .configure(messages) // Tweets related routes
            .configure(user) // User related routes
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
