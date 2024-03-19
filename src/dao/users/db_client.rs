use super::dto::{CreateDto, ListParamsDto, UpdateDto, UserDto, UsersListDto};
use super::DaoTrait;
use crate::dao::db_client::DatabaseClient;
use axum::async_trait;
use chrono::Utc;
use database::errors::{CreateError, DeleteError, GetError, ListError, UpdateError};
use entities::users::{ActiveModel, Model};
use sea_orm::ActiveValue;
use uuid::Uuid;

#[async_trait]
impl DaoTrait for DatabaseClient {
    async fn create_user(&self, payload: CreateDto) -> Result<UserDto, CreateError> {
        let model: Model = payload.into();
        self.client.create_user(model).await.map(Into::into)
    }

    async fn list_users(&self, params: ListParamsDto) -> Result<UsersListDto, ListError> {
        let users = self
            .client
            .list_users(params.predicate)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(UsersListDto { items: users })
    }

    async fn get_user(&self, id: Uuid) -> Result<UserDto, GetError> {
        self.client.get_user(id).await.map(Into::into)
    }

    async fn update_user(&self, id: Uuid, payload: UpdateDto) -> Result<UserDto, UpdateError> {
        let active_model: ActiveModel = ActiveModel {
            id: ActiveValue::Set(id),
            name: ActiveValue::Set(payload.name),
            updated_at: ActiveValue::Set(Utc::now().naive_utc()),
            ..Default::default()
        };
        self.client
            .update_user(id, active_model)
            .await
            .map(Into::into)
    }

    async fn delete_user(&self, id: Uuid) -> Result<(), DeleteError> {
        self.client.delete_user(id).await
    }
}
