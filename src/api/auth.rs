use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::models::init::Tweetbook;

#[derive(Deserialize)]
struct Auth {
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
    HttpResponse::Ok().body("Signin!")
}
