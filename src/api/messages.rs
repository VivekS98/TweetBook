use actix_web::{get, post, web, Either, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::{
    models::{init::Tweetbook, messages::Message},
    utils::{auth::Authorization, error::UserError},
};

#[derive(Clone, Deserialize)]
pub struct MessageInput {
    pub text: String,
}

pub fn messages(cfg: &mut web::ServiceConfig) {
    cfg.service(all_tweets).service(post_tweet);
}

#[get("/api/tweets")]
async fn all_tweets(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;
    match id_res {
        Ok(_) => {
            let messages = Message::get_all_messages(db).await;

            match messages {
                Ok(msgs) => Either::Left(HttpResponse::Ok().json(msgs)),
                Err(_) => Either::Right(Err(UserError::UserNotExists)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}

#[post("/api/users/tweet")]
async fn post_tweet(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    body: web::Json<MessageInput>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;
    match id_res {
        Ok(id) => {
            let message = Message::insert_message(db, body.text.to_owned(), id.to_string()).await;

            match message {
                Ok(msg) => Either::Left(HttpResponse::Ok().json(msg)),
                Err(_) => Either::Right(Err(UserError::InternalServerError)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}
