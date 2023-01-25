use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum UserError {
    #[display(fmt = "Unauthorised access. Please signup or login.")]
    Unauthorised,
    #[display(fmt = "User doesn't exist!")]
    UserNotExists,
    #[display(fmt = "User already exists!")]
    UserAlreadyExists,
    #[display(fmt = "Something went wrong! Please try again later.")]
    InternalServerError,
    #[display(fmt = "Wrong Email or Password. PLease Try with the valid credentials.")]
    WrongEmailOrPassword,
    #[display(fmt = "Something's wrong with your request's info! Please check and try again.")]
    WrongInfo,
}

impl error::ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::UserNotExists => StatusCode::BAD_REQUEST,
            UserError::UserAlreadyExists => StatusCode::NOT_ACCEPTABLE,
            UserError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::WrongEmailOrPassword => StatusCode::BAD_REQUEST,
            UserError::Unauthorised => StatusCode::UNAUTHORIZED,
            UserError::WrongInfo => StatusCode::NOT_ACCEPTABLE,
        }
    }
}
