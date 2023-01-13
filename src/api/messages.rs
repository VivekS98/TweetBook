use actix_web::{get, web, Either, HttpResponse};

use crate::{
    models::{init::Tweetbook, messages::Message},
    utils::error::UserError,
};

pub fn messages(cfg: &mut web::ServiceConfig) {
    cfg.service(all_tweets);
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
