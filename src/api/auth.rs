use actix_web::{post, web, Either, HttpRequest, HttpResponse};
use bcrypt::verify;
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

use crate::{
    models::{
        init::Tweetbook,
        users::{MinUser, User},
    },
    utils::{auth::Authorization, error::UserError},
};

#[derive(Clone, Deserialize)]
pub struct AuthCredentials {
    pub username: Option<String>,
    pub email: String,
    pub password: String,
    pub ip: Option<IpAddr>,
}

#[derive(Serialize)]
struct AuthResponse {
    id: ObjectId,
    username: String,
    #[serde(rename = "profileImgUrl")]
    profile_img_url: String,
    token: String,
}

pub fn auth(cfg: &mut web::ServiceConfig) {
    cfg.service(signup).service(signin);
}

#[post("/api/auth/signup")]
async fn signup(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    body: web::Json<AuthCredentials>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let user_data = User::get_user_by_email(db.clone(), &body.email).await;

    match user_data {
        Ok(old_users) => {
            if old_users.len() > 0 {
                Either::Right(Err(UserError::UserAlreadyExists))
            } else {
                let inserted = User::add_user(
                    db,
                    AuthCredentials {
                        username: body.username.clone(),
                        email: body.email.clone(),
                        password: body.password.clone(),
                        ip: Some(req.peer_addr().unwrap().ip()),
                    },
                )
                .await;

                match inserted {
                    Ok(new_user) => {
                        let token = Authorization::get_token(new_user.clone());

                        Either::Left(HttpResponse::Ok().json(AuthResponse {
                            id: new_user.id,
                            username: new_user.username,
                            profile_img_url: new_user.profile_img_url.unwrap_or_default(),
                            token,
                        }))
                    }
                    Err(_) => Either::Right(Err(UserError::InternalServerError)),
                }
            }
        }
        Err(_) => Either::Right(Err(UserError::InternalServerError)),
    }
}

#[post("/api/auth/signin")]
async fn signin(
    req: HttpRequest,
    db: web::Data<Tweetbook>,
    json: web::Json<AuthCredentials>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let user_data = User::get_user_by_email(db.clone(), json.email.as_str()).await;

    match user_data {
        Ok(mut old_users) => {
            if old_users.len() > 0 {
                let user = old_users.remove(0);
                let matched = verify(json.password.as_str(), user.password.unwrap().as_str());

                match matched {
                    Ok(password_match) => {
                        if password_match == true {
                            let ip_exists = user
                                .active_ips
                                .iter()
                                .any(|ip| ip.contains(&req.peer_addr().unwrap().ip()));

                            if !ip_exists {
                                User::update_user(
                                    db,
                                    user.id.to_string(),
                                    doc! { "$push": { "activeIps": req.peer_addr().unwrap().ip().to_string() }},
                                )
                                .await
                                .unwrap();
                            }

                            let claims = MinUser {
                                id: user.id,
                                username: user.username,
                                email: user.email,
                                profile_img_url: user.profile_img_url,
                            };

                            let token = Authorization::get_token(claims.clone());

                            Either::Left(HttpResponse::Ok().json(AuthResponse {
                                id: claims.id,
                                username: claims.username,
                                profile_img_url: claims.profile_img_url.unwrap_or_default(),
                                token,
                            }))
                        } else {
                            Either::Right(Err(UserError::WrongEmailOrPassword))
                        }
                    }
                    Err(_) => Either::Right(Err(UserError::WrongEmailOrPassword)),
                }
            } else {
                Either::Right(Err(UserError::WrongEmailOrPassword))
            }
        }
        Err(_) => Either::Right(Err(UserError::WrongEmailOrPassword)),
    }
}
