pub mod db_client;
pub mod dto;

use axum::async_trait;
use uuid::Uuid;

use database::errors::{CreateError, DeleteError, GetError, ListError, UpdateError};

use crate::dao::users::dto::{CreateDto, ListParamsDto, UpdateDto, UserDto, UsersListDto};

#[async_trait]
pub trait DaoTrait {
    async fn create(&self, payload: CreateDto) -> Result<UserDto, CreateError>;

    async fn list(&self, params: ListParamsDto) -> Result<UsersListDto, ListError>;

    async fn get(&self, id: Uuid) -> Result<UserDto, GetError>;

    async fn update(&self, id: Uuid, payload: UpdateDto) -> Result<UserDto, UpdateError>;

    async fn delete(&self, id: Uuid) -> Result<(), DeleteError>;
}
