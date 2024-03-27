use color_eyre::Report as AnyError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("Item with id {id:?} already exist in database")]
    AlreadyExist { id: Uuid },
    #[error("Unexpected error during item creation: {error}")]
    Unexpected { error: AnyError },
}

#[derive(Error, Debug)]
pub enum ListError {
    #[error("Unexpected error during list collection: {error}")]
    Unexpected { error: AnyError },
}

#[derive(Error, Debug)]
pub enum GetError {
    #[error("Item with id {id:?} not found in database")]
    NotFound { id: Uuid },
    #[error("Unexpected error during {id:?} item collection: {error}")]
    Unexpected { id: Uuid, error: AnyError },
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Item with id {id:?} not found in database")]
    NotFound { id: Uuid },
    #[error("Unexpected error during {id:?} item update: {error}")]
    Unexpected { id: Uuid, error: AnyError },
}

#[derive(Error, Debug)]
pub enum DeleteError {
    #[error("Item with id {id:?} not found in database")]
    NotFound { id: Uuid },
    #[error("Unexpected error during {id:?} item deletion: {error}")]
    Unexpected { id: Uuid, error: AnyError },
}

#[derive(Error, Debug)]
pub enum HealthcheckError {
    #[error("Unexpected error during healthcheck: {error}")]
    Unexpected { error: AnyError },
}

impl From<GetError> for UpdateError {
    fn from(value: GetError) -> Self {
        match value {
            GetError::NotFound { id } => UpdateError::NotFound { id },
            GetError::Unexpected { id, error } => UpdateError::Unexpected { id, error },
        }
    }
}
