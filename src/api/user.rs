use actix_web::{get, web, Either, HttpRequest, HttpResponse};

use crate::{
    models::{init::Tweetbook, users::User},
    utils::{auth::Authorization, error::UserError},
};

pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(user_tweets);
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
