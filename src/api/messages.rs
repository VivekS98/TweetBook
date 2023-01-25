use std::str::FromStr;

use actix_web::{delete, get, post, web, Either, HttpRequest, HttpResponse};
use mongodb::bson::{doc, oid::ObjectId};
use serde::Deserialize;

use crate::{
    models::{init::Tweetbook, messages::Message},
    utils::{auth::Authorization, error::UserError},
};

#[derive(Clone, Deserialize)]
struct MessageInput {
    text: String,
}

pub fn messages(cfg: &mut web::ServiceConfig) {
    cfg.service(all_tweets)
        .service(post_tweet)
        .service(like_tweet)
        .service(unlike_tweet)
        .service(get_tweet)
        .service(delete_tweet);
}

#[get("/api/tweets")]
async fn all_tweets(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;
    match id_res {
        Ok(_) => {
            let messages = Message::get_message_by_query::<Message>(db, None).await;

            match messages {
                Ok(msgs) => Either::Left(HttpResponse::Ok().json(msgs)),
                Err(_) => Either::Right(Err(UserError::InternalServerError)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}

#[post("/api/user/tweet")]
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

#[post("/api/user/tweet/{tweet_id}/like")]
async fn like_tweet(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    path: web::Path<String>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;
    match id_res {
        Ok(id) => {
            let tweet_id = path.into_inner();
            let message = Message::update_message(
                db,
                tweet_id,
                doc! { "$addToSet": { "likes": { "$each": vec![id]}  }},
            )
            .await;

            match message {
                Ok(msg) => Either::Left(HttpResponse::Ok().json(msg)),
                Err(_) => Either::Right(Err(UserError::InternalServerError)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}

#[delete("/api/user/tweet/{tweet_id}/like")]
async fn unlike_tweet(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    path: web::Path<String>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;
    match id_res {
        Ok(id) => {
            let tweet_id = path.into_inner();
            let message = Message::update_message(
                db,
                tweet_id,
                doc! { "$pull": { "likes": { "$in": vec![id]} }},
            )
            .await;

            match message {
                Ok(msg) => Either::Left(HttpResponse::Ok().json(msg)),
                Err(_) => Either::Right(Err(UserError::InternalServerError)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}

#[get("/api/user/tweet/{tweet_id}")]
async fn get_tweet(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    path: web::Path<String>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;
    match id_res {
        Ok(_id) => {
            let tweet_id = path.into_inner();
            let message = Message::get_message_by_query::<Message>(
                db,
                Some(doc! {
                    "$match": {
                        "_id": ObjectId::from_str(&tweet_id).unwrap()
                    }
                }),
            )
            .await;

            println!("{:?}", message);

            match message {
                Ok(mut msg) => Either::Left(HttpResponse::Ok().json(msg.remove(0))),
                Err(_) => Either::Right(Err(UserError::InternalServerError)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}

#[delete("/api/user/tweet/{tweet_id}")]
async fn delete_tweet(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    path: web::Path<String>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;
    match id_res {
        Ok(id) => {
            let tweet_id = path.into_inner();
            let message = Message::delete_message(db, tweet_id, id.to_string()).await;

            println!("{:?}", message);

            match message {
                Ok(_) => Either::Left(HttpResponse::Ok().body(format!("Tweet deleted!"))),
                Err(_) => Either::Right(Err(UserError::InternalServerError)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}
