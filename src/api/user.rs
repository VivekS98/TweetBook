use actix_web::{delete, get, post, web, Either, HttpRequest, HttpResponse};
use mongodb::bson::doc;

use crate::{
    models::{init::Tweetbook, users::User},
    utils::{auth::Authorization, error::UserError},
};

pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(user_tweets)
        .service(follow_user)
        .service(unfollow_user);
}

#[get("/api/user/profile")]
async fn user_tweets(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;

    match id_res {
        Ok(id) => {
            let users_response = User::get_user_details(db, id.to_string()).await;

            match users_response {
                Ok(users) => Either::Left(HttpResponse::Ok().json(users)),
                Err(_) => Either::Right(Err(UserError::UserNotExists)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}

#[post("/api/users/follow/{user_id}")]
async fn follow_user(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    path: web::Path<String>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;

    match id_res {
        Ok(id) => {
            let user_id = path.into_inner();
            let user_res = User::get_user_details(db.clone(), user_id).await;

            match user_res {
                Ok(mut users) => {
                    if users.len() > 0 {
                        let user = users.remove(0);

                        let user_follower = User::update_user(
                            db.clone(),
                            id.to_string(),
                            doc! { "$addToSet": { "following": { "$each": vec![user.id]}  }},
                        )
                        .await
                        .unwrap();

                        User::update_user(
                            db,
                            user.id.to_string(),
                            doc! { "$addToSet": { "followers": { "$each": vec![user_follower.id]}}},
                        )
                        .await
                        .unwrap();

                        Either::Left(HttpResponse::Ok().json(user))
                    } else {
                        Either::Right(Err(UserError::UserNotExists))
                    }
                }
                Err(_) => Either::Right(Err(UserError::UserNotExists)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}

#[delete("/api/users/follow/{user_id}")]
async fn unfollow_user(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    path: web::Path<String>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;

    match id_res {
        Ok(id) => {
            let user_id = path.into_inner();
            let user_res = User::get_user_details(db.clone(), user_id).await;

            match user_res {
                Ok(mut users) => {
                    if users.len() > 0 {
                        let user = users.remove(0);

                        let user_follower = User::update_user(
                            db.clone(),
                            id.to_string(),
                            doc! { "$pull": { "following": { "$in": vec![user.id]}  }},
                        )
                        .await
                        .unwrap();

                        User::update_user(
                            db,
                            user.id.to_string(),
                            doc! { "$pull": { "followers": { "$in": vec![user_follower.id]}}},
                        )
                        .await
                        .unwrap();

                        Either::Left(HttpResponse::Ok().json(user))
                    } else {
                        Either::Right(Err(UserError::UserNotExists))
                    }
                }
                Err(_) => Either::Right(Err(UserError::UserNotExists)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}
