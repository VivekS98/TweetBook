use actix_web::{get, post, web, Either, HttpResponse};
use serde::Deserialize;

use crate::{
    models::{init::Tweetbook, messages::Message},
    utils::error::UserError,
};

#[derive(Clone, Deserialize)]
pub struct MessageInput {
    pub text: String,
}

pub fn messages(cfg: &mut web::ServiceConfig) {
    cfg.service(all_tweets).service(post_tweet);
}

#[get("/api/messages")]
async fn all_tweets(
    db: web::Data<Tweetbook>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let messages = Message::get_all_messages(db).await;

    match messages {
        Ok(msgs) => Either::Left(HttpResponse::Ok().json(msgs)),
        Err(_) => Either::Right(Err(UserError::UserNotExists)),
    }
}

#[post("/api/users/{id}/messages")]
async fn post_tweet(
    db: web::Data<Tweetbook>,
    path: web::Path<String>,
    body: web::Json<MessageInput>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let message = Message::insert_message(db, body.text.as_str(), path.into_inner()).await;

    match message {
        Ok(msg) => Either::Left(HttpResponse::Ok().json(msg)),
        Err(_) => Either::Right(Err(UserError::InternalServerError)),
    }
}
