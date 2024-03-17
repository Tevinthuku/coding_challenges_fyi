use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::Display;
use serde_json::json;

#[derive(Debug, Display)]
pub enum ApiError {
    #[display(fmt = "internal error")]
    InternalError(anyhow::Error),

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "not found")]
    NotFound,
}

impl error::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(json!({
                "error": self.to_string()
            }))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadClientData => StatusCode::BAD_REQUEST,
            ApiError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}
