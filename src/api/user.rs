use actix_web::{delete, get, post, put, web, Either, HttpRequest, HttpResponse};
use mongodb::bson::doc;
use serde::Deserialize;

use crate::{
    models::{
        init::Tweetbook,
        users::{MinUser, User},
    },
    utils::{auth::Authorization, error::UserError},
};

pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(user_profile)
        .service(update_profile)
        .service(follow_user)
        .service(unfollow_user)
        .service(user_search);
}

#[derive(Deserialize)]
struct UserSearch {
    search: String,
}

#[derive(Deserialize)]
struct UserUpdate {
    bio: String,
    #[serde(rename = "profileImgUrl")]
    profile_img_url: String,
}

#[get("/api/user/profile/{user_id}")]
async fn user_profile(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    path: web::Path<String>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;

    match id_res {
        Ok(_) => {
            let user_id = path.into_inner();
            let users_response = User::get_user_details(db, user_id).await;

            match users_response {
                Ok(users) => Either::Left(HttpResponse::Ok().json(users)),
                Err(_) => Either::Right(Err(UserError::UserNotExists)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}

#[put("/api/user/profile")]
async fn update_profile(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    body: web::Json<UserUpdate>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;

    match id_res {
        Ok(id) => {
            let users_response = User::update_user(
                db,
                id.to_string(),
                doc! {"$set": {"bio": body.bio.to_owned(), "profileImgUrl": body.profile_img_url.to_owned()}},
            )
            .await;

            match users_response {
                Ok(users) => Either::Left(HttpResponse::Ok().json(users)),
                Err(_) => Either::Right(Err(UserError::UserNotExists)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}

#[post("/api/user/follow/{user_id}")]
async fn follow_user(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    path: web::Path<String>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;

    match id_res {
        Ok(id) => {
            let user_id = path.into_inner();
            let user_res = User::get_user_by_query::<MinUser>(
                db.clone(),
                doc! {
                    "$match": {
                        "$expr": {
                            "$eq": ["$_id", {"$toObjectId": user_id}]
                        }
                    }
                },
            )
            .await;

            match user_res {
                Ok(mut users) => {
                    if users.len() > 0 {
                        let user = users.remove(0);

                        let user_follower = User::update_user(
                            db.clone(),
                            id.to_string(),
                            doc! { "$addToSet": { "following": { "$each": vec![user.id]} }},
                        )
                        .await
                        .unwrap();

                        User::update_user(
                            db.clone(),
                            user.id.to_string(),
                            doc! { "$addToSet": { "followers": { "$each": vec![user_follower.id]}}},
                        )
                        .await
                        .unwrap();

                        let mut user_updated = User::get_user_details(db, user.id.to_string())
                            .await
                            .unwrap();

                        Either::Left(HttpResponse::Ok().json(user_updated.remove(0)))
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

#[delete("/api/user/follow/{user_id}")]
async fn unfollow_user(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    path: web::Path<String>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;

    match id_res {
        Ok(id) => {
            let user_id = path.into_inner();
            let user_res = User::get_user_by_query::<MinUser>(
                db.clone(),
                doc! {
                    "$match": {
                        "$expr": {
                            "$eq": ["$_id", {"$toObjectId": user_id}]
                        }
                    }
                },
            )
            .await;

            match user_res {
                Ok(mut users) => {
                    if users.len() > 0 {
                        let user = users.remove(0);

                        let user_follower = User::update_user(
                            db.clone(),
                            id.to_string(),
                            doc! { "$pull": { "following": { "$in": vec![user.id]} }},
                        )
                        .await
                        .unwrap();

                        User::update_user(
                            db.clone(),
                            user.id.to_string(),
                            doc! { "$pull": { "followers": { "$in": vec![user_follower.id]}}},
                        )
                        .await
                        .unwrap();

                        let mut user_updated = User::get_user_details(db, user.id.to_string())
                            .await
                            .unwrap();

                        Either::Left(HttpResponse::Ok().json(user_updated.remove(0)))
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

#[get("/api/users")]
async fn user_search(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    info: web::Query<UserSearch>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id_res = Authorization::verify_request(req).await;

    match id_res {
        Ok(_) => {
            let users_response = User::get_user_by_query::<MinUser>(
                db,
                doc! {
                    "$match": {
                        "username": {
                            "$regex": info.search.to_string(),
                            "$options": "i"
                        }
                    }
                },
            )
            .await;

            match users_response {
                Ok(users) => Either::Left(HttpResponse::Ok().json(users)),
                Err(_) => Either::Right(Err(UserError::UserNotExists)),
            }
        }
        Err(_) => Either::Right(Err(UserError::Unauthorised)),
    }
}
