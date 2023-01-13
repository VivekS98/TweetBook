use actix_web::{get, web, Either, HttpResponse};

use crate::{
    models::{init::Tweetbook, users::User},
    utils::error::UserError,
};

pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(user_tweets);
}

#[get("/api/users/{id}")]
async fn user_tweets(
    db: web::Data<Tweetbook>,
    path: web::Path<String>,
) -> Either<HttpResponse, Result<&'static str, UserError>> {
    let id = path.into_inner();
    let users_response = User::get_user_details(db, &id).await;

    match users_response {
        Ok(users) => {
            if users.len() > 0 {
                Either::Left(HttpResponse::Ok().json(users))
            } else {
                Either::Right(Err(UserError::UserNotExists))
            }
        }
        Err(_) => Either::Right(Err(UserError::InternalServerError)),
    }
}
