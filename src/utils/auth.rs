use actix_web::{http::header, web::Data, HttpRequest};
use chrono::{Months, Utc};
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

use crate::models::{
    init::Tweetbook,
    users::{MinUser, User},
};

use super::error::UserError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Authorization {
    sub: String,
    company: String,
    exp: usize,
}

impl Authorization {
    pub fn get_token(user: MinUser) -> String {
        dotenv().ok();
        let _secret = env::var("TOKEN_SECRET").unwrap();

        let expiration = Utc::now()
            .checked_add_months(Months::new(12))
            .unwrap()
            .timestamp();

        let _claims = Self {
            sub: user.id.to_string(),
            company: "TweetBook".to_string(),
            exp: expiration as usize,
        };

        encode(
            &Header::default(),
            &_claims,
            &EncodingKey::from_secret(_secret.as_ref()),
        )
        .unwrap()
    }

    pub async fn verify_request(
        req: HttpRequest,
    ) -> Result<mongodb::bson::oid::ObjectId, UserError> {
        let headers = req.headers();
        let auth_token = headers.get(header::AUTHORIZATION);

        match auth_token {
            Some(token) => {
                let secret = env::var("TOKEN_SECRET").unwrap();

                let decoded = decode::<Self>(
                    token.to_str().unwrap(),
                    &DecodingKey::from_secret(secret.as_ref()),
                    &Validation::new(Algorithm::HS256),
                );

                match decoded {
                    Ok(token_data) => {
                        let data = req.app_data::<Data<Tweetbook>>().unwrap().to_owned();
                        let user_res = User::get_user_by_id(data, token_data.claims.sub).await;

                        match user_res {
                            Ok(user) => {
                                let ip_exists = user[0]
                                    .active_ips
                                    .iter()
                                    .any(|ip| ip.contains(&req.peer_addr().unwrap().ip()));

                                if ip_exists {
                                    Ok(user[0].id)
                                } else {
                                    Err(UserError::Unauthorised)
                                }
                            }
                            Err(_) => Err(UserError::Unauthorised),
                        }
                    }
                    Err(_) => Err(UserError::Unauthorised),
                }
            }
            None => Err(UserError::Unauthorised),
        }
    }
}
