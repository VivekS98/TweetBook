use actix_web::{get, web, HttpResponse, Responder};

use crate::models::{init::Tweetbook, users::User};

pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(user_tweets);
}

#[get("/api/users/{id}")]
async fn user_tweets(db: web::Data<Tweetbook>, path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let users = User::get_users(db, &id).await;

    HttpResponse::Ok().json(users)
}
