pub mod db_client;
pub mod dto;

use axum::async_trait;
use uuid::Uuid;

use database::errors::{CreateError, DeleteError, GetError, ListError, UpdateError};

use dto::{CreateDto, ListParamsDto, UpdateDto, UserDto, UsersListDto};

#[async_trait]
pub trait DaoTrait {
    async fn create_user(&self, payload: CreateDto) -> Result<UserDto, CreateError>;

    async fn list_users(&self, params: ListParamsDto) -> Result<UsersListDto, ListError>;

    async fn get_user(&self, id: Uuid) -> Result<UserDto, GetError>;

    async fn update_user(&self, id: Uuid, payload: UpdateDto) -> Result<UserDto, UpdateError>;

    async fn delete_user(&self, id: Uuid) -> Result<(), DeleteError>;
}
