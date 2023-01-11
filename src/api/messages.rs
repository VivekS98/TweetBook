use actix_web::{get, web, HttpResponse, Responder};

use crate::models::{init::Tweetbook, messages::Message};

pub fn messages(cfg: &mut web::ServiceConfig) {
    cfg.service(all_tweets);
}

#[get("/api/messages")]
async fn all_tweets(db: web::Data<Tweetbook>) -> impl Responder {
    let messages = Message::get_all_messages(db).await;
    HttpResponse::Ok().json(messages)
}
