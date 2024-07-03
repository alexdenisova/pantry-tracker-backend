use axum::response::{IntoResponse, Response};
use axum::Json;
use color_eyre::Report as AnyError;
use http::StatusCode;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::database::errors::{CreateError, DeleteError, GetError, ListError, UpdateError};

#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    AlreadyExists { id: Uuid },
    NotFound { id: String },
    UnprocessableEntity { error: AnyError },
    Other { error: AnyError },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // How we want errors responses to be serialized
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status, message) = match self {
            AppError::Unauthorized => {
                log::info!("User unauthorized");
                (StatusCode::UNAUTHORIZED, "User is unauthorized".to_owned())
            }
            AppError::AlreadyExists { id } => (
                StatusCode::CONFLICT,
                format!("Item with id {id} already exists"),
            ),
            AppError::NotFound { id } => (
                StatusCode::NOT_FOUND,
                format!("Item with id {id} not found"),
            ),
            AppError::UnprocessableEntity { error } => {
                (StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
            }
            AppError::Other { error: _ } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong".to_owned(),
            ),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

impl From<CreateError> for AppError {
    fn from(val: CreateError) -> Self {
        log::error!("{}", val);
        match val {
            CreateError::AlreadyExist { id } => AppError::AlreadyExists { id },
            CreateError::Unexpected { error } => AppError::Other { error },
        }
    }
}

impl From<ListError> for AppError {
    fn from(val: ListError) -> Self {
        log::error!("{}", val);
        match val {
            ListError::Unexpected { error } => AppError::Other { error },
            ListError::Unprocessable { error } => AppError::UnprocessableEntity { error },
        }
    }
}

impl From<GetError> for AppError {
    fn from(val: GetError) -> Self {
        log::error!("{}", val);
        match val {
            GetError::NotFound { id } => AppError::NotFound { id: id.to_string() },
            GetError::Unexpected { id: _, error } => AppError::Other { error },
        }
    }
}

impl From<UpdateError> for AppError {
    fn from(val: UpdateError) -> Self {
        log::error!("{}", val);
        match val {
            UpdateError::NotFound { id } => AppError::NotFound { id: id.to_string() },
            UpdateError::Unexpected { id: _, error } => AppError::Other { error },
        }
    }
}

impl From<DeleteError> for AppError {
    fn from(val: DeleteError) -> Self {
        log::error!("{}", val);
        match val {
            DeleteError::NotFound { id } => AppError::NotFound { id: id.to_string() },
            DeleteError::Unexpected { id: _, error } => AppError::Other { error },
        }
    }
}

#[derive(Error, Debug)]
pub enum VerifyError {
    #[error("Incorrect username or password")]
    Unauthorized,
    #[error("User {user_id} not found")]
    NotFound { user_id: Uuid },
    #[error("{error}")]
    Other { error: AnyError },
}

impl From<VerifyError> for AppError {
    fn from(val: VerifyError) -> Self {
        log::error!("{}", val);
        match val {
            VerifyError::Unauthorized => AppError::Unauthorized,
            VerifyError::NotFound { user_id } => AppError::NotFound {
                id: user_id.to_string(),
            },
            VerifyError::Other { error } => AppError::Other { error },
        }
    }
}

impl From<GetError> for VerifyError {
    fn from(val: GetError) -> Self {
        log::error!("{}", val);
        if let GetError::NotFound { id } = val {
            return VerifyError::NotFound { user_id: id };
        }
        VerifyError::Other { error: val.into() }
    }
}
