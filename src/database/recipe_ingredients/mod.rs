pub mod dto;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, Set};
use uuid::Uuid;

use self::dto::{
    CreateDto, ListParamsDto, RecipeIngredientDto, RecipeIngredientsListDto, UpdateDto,
};
use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError, UpdateError},
    DBClient,
};
use db_entities::recipe_ingredients::{ActiveModel, Column, Entity, Model};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_recipe_ingredient(
        &self,
        request: CreateDto,
    ) -> Result<RecipeIngredientDto, CreateError>;
    async fn get_recipe_ingredient(&self, id: Uuid) -> Result<RecipeIngredientDto, GetError>;
    async fn list_recipe_ingredients(
        &self,
        list_params: ListParamsDto,
    ) -> Result<RecipeIngredientsListDto, ListError>;
    async fn update_recipe_ingredient(
        &self,
        id: Uuid,
        request: UpdateDto,
    ) -> Result<RecipeIngredientDto, UpdateError>;
    async fn delete_recipe_ingredient(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_recipe_ingredient(
        &self,
        request: CreateDto,
    ) -> Result<RecipeIngredientDto, CreateError> {
        let model: Model = request.into();
        let id = model.id;
        let active_model: ActiveModel = model.into();
        Ok(active_model
            .insert(&self.database_connection)
            .await
            .map_err(|err| {
                if let DbErr::RecordNotInserted = err {
                    CreateError::AlreadyExist { id }
                } else {
                    CreateError::Unexpected { error: err.into() }
                }
            })?
            .into())
    }
    async fn get_recipe_ingredient(&self, id: Uuid) -> Result<RecipeIngredientDto, GetError> {
        Ok(Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map_err(|err| GetError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(GetError::NotFound { id })?
            .into())
    }
    async fn list_recipe_ingredients(
        &self,
        list_params: ListParamsDto,
    ) -> Result<RecipeIngredientsListDto, ListError> {
        let mut entity = match list_params.recipe_id {
            Some(value) => Entity::find().filter(Column::RecipeId.eq(value)),
            None => Entity::find(),
        };
        entity = match list_params.ingredient_id {
            Some(value) => entity.filter(Column::IngredientId.eq(value)),
            None => entity,
        };
        Ok(RecipeIngredientsListDto {
            items: entity
                .order_by_desc(Column::UpdatedAt)
                .all(&self.database_connection)
                .await
                .map_err(|err| ListError::Unexpected { error: err.into() })?
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
    async fn update_recipe_ingredient(
        &self,
        id: Uuid,
        request: UpdateDto,
    ) -> Result<RecipeIngredientDto, UpdateError> {
        let recipe_ingredient: Model = Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map_err(|err| UpdateError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(UpdateError::NotFound { id })?;
        let mut recipe_ingredient: ActiveModel = recipe_ingredient.into();
        recipe_ingredient.ingredient_id = Set(request.ingredient_id);
        recipe_ingredient.amount = Set(request.amount);
        recipe_ingredient.unit = Set(request.unit);
        recipe_ingredient.optional = Set(request.optional);
        recipe_ingredient.updated_at = Set(Utc::now().naive_utc());

        Ok(Entity::update(recipe_ingredient)
            .filter(Column::Id.eq(id))
            .exec(&self.database_connection)
            .await
            .map_err(|err| {
                if let DbErr::RecordNotUpdated = err {
                    UpdateError::NotFound { id }
                } else {
                    UpdateError::Unexpected {
                        id,
                        error: err.into(),
                    }
                }
            })?
            .into())
    }
    async fn delete_recipe_ingredient(&self, id: Uuid) -> Result<(), DeleteError> {
        if Entity::delete_by_id(id)
            .exec(&self.database_connection)
            .await
            .map_err(|err| DeleteError::Unexpected {
                id,
                error: err.into(),
            })?
            .rows_affected
            == 0
        {
            Err(DeleteError::NotFound { id })
        } else {
            Ok(())
        }
    }
}
