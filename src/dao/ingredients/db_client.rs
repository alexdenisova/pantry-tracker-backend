use super::dto::{CreateDto, IngredientDto, IngredientsListDto, ListParamsDto, UpdateDto};
use super::DaoTrait;
use crate::dao::db_client::DatabaseClient;
use axum::async_trait;
use database::errors::{CreateError, DeleteError, GetError, ListError, UpdateError};
use entities::ingredients::{ActiveModel, Model};
use sea_orm::ActiveValue;
use uuid::Uuid;

#[async_trait]
impl DaoTrait for DatabaseClient {
    async fn create_ingredient(&self, payload: CreateDto) -> Result<IngredientDto, CreateError> {
        let model: Model = payload.into();
        self.client.create_ingredient(model).await.map(Into::into)
    }

    async fn list_ingredients(
        &self,
        params: ListParamsDto,
    ) -> Result<IngredientsListDto, ListError> {
        let users = self
            .client
            .list_ingredients(params.predicate)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(IngredientsListDto { items: users })
    }

    async fn get_ingredient(&self, id: Uuid) -> Result<IngredientDto, GetError> {
        self.client.get_ingredient(id).await.map(Into::into)
    }

    async fn update_ingredient(
        &self,
        id: Uuid,
        payload: UpdateDto,
    ) -> Result<IngredientDto, UpdateError> {
        let active_model: ActiveModel = ActiveModel {
            id: ActiveValue::Set(id),
            name: ActiveValue::Set(payload.name),
            can_be_eaten_raw: ActiveValue::Set(payload.can_be_eaten_raw),
            ..Default::default()
        };
        self.client
            .update_ingredient(id, active_model)
            .await
            .map(Into::into)
    }

    async fn delete_ingredient(&self, id: Uuid) -> Result<(), DeleteError> {
        self.client.delete_ingredient(id).await
    }
}
