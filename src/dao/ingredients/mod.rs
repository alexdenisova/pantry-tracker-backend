pub mod db_client;
pub mod dto;

use axum::async_trait;
use uuid::Uuid;

use database::errors::{CreateError, DeleteError, GetError, ListError, UpdateError};

use dto::{CreateDto, IngredientDto, IngredientsListDto, ListParamsDto, UpdateDto};

#[async_trait]
pub trait DaoTrait {
    async fn create_ingredient(&self, payload: CreateDto) -> Result<IngredientDto, CreateError>;

    async fn list_ingredients(
        &self,
        params: ListParamsDto,
    ) -> Result<IngredientsListDto, ListError>;

    async fn get_ingredient(&self, id: Uuid) -> Result<IngredientDto, GetError>;

    async fn update_ingredient(
        &self,
        id: Uuid,
        payload: UpdateDto,
    ) -> Result<IngredientDto, UpdateError>;

    async fn delete_ingredient(&self, id: Uuid) -> Result<(), DeleteError>;
}
