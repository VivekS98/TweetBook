use actix_web::{post, web, Either, HttpResponse, Responder};
use bcrypt::verify;
use dotenv::dotenv;
use jsonwebtoken::{encode, EncodingKey, Header};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::env;

use crate::{
    models::{
        init::Tweetbook,
        users::{MinUser, User},
    },
    utils::error::UserError,
};

#[derive(Deserialize)]
pub struct AuthCredentials {
    pub email: String,
    pub password: String,
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
async fn signup(db: web::Data<Tweetbook>, body: web::Json<AuthCredentials>) -> impl Responder {
    HttpResponse::Ok().body("Signup!")
}

#[post("/api/auth/signin")]
async fn signin(
    db: web::Data<Tweetbook>,
    json: web::Json<AuthCredentials>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let user_data = User::get_user_by_email(db, json.email.as_str()).await;

    match user_data {
        Ok(mut users) => {
            let user = Box::new(users.remove(0));
            let matched = verify(json.password.as_str(), user.password.unwrap().as_str());

            match matched {
                Ok(password_match) => {
                    if password_match == true {
                        dotenv().ok();
                        let secret = env::var("TOKEN_SECRET").unwrap();
                        let claims = MinUser {
                            id: user.id,
                            username: user.username,
                            email: user.email,
                            bio: user.bio,
                            profile_img_url: user.profile_img_url,
                        };

                        let token = encode(
                            &Header::default(),
                            &claims,
                            &EncodingKey::from_secret(secret.as_ref()),
                        )
                        .unwrap();

                        Either::Left(HttpResponse::Ok().json(AuthResponse {
                            id: claims.id,
                            username: claims.username,
                            profile_img_url: claims.profile_img_url,
                            token,
                        }))
                    } else {
                        Either::Right(Err(UserError::WrongEmailOrPassword))
                    }
                }
                Err(_) => Either::Right(Err(UserError::WrongEmailOrPassword)),
            }
        }
        Err(_) => Either::Right(Err(UserError::WrongEmailOrPassword)),
    }
}
