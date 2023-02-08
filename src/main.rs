mod api;
mod models;
mod utils;

use actix_web::{web, App, HttpServer};
use actix_web_lab::web::spa;
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
            .service(
                spa()
                    .index_file("./public/index.html")
                    .static_resources_mount("/static")
                    .static_resources_location("./public/static/")
                    .finish(),
            )
    })
    .bind(("0.0.0.0", 8088))?
    .run()
    .await
}
