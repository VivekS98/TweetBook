use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::models::{init::Tweetbook, users::User};

#[derive(Deserialize)]
pub struct Auth {
    username: String,
    password: String,
}

pub fn auth(cfg: &mut web::ServiceConfig) {
    cfg.service(signup).service(signin);
}

#[post("/api/auth/signup")]
async fn signup(db: web::Data<Tweetbook>, body: web::Json<Auth>) -> impl Responder {
    HttpResponse::Ok().body("Signup!")
}

#[post("/api/auth/signin")]
async fn signin(db: web::Data<Tweetbook>, body: web::Json<Auth>) -> impl Responder {
    let user = User::get_user(db, body).await;
    HttpResponse::Ok().body("Signin!")
}
