use async_trait::async_trait;
use chrono::Utc;
use db_entities::recipes::{ActiveModel, Column, Entity, Model};
use sea_orm::*;
use uuid::Uuid;

pub mod dto;

use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError, UpdateError},
    DBClient,
};

use self::dto::{CreateDto, ListParamsDto, RecipeDto, RecipesListDto, UpdateDto};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_recipe(&self, request: CreateDto) -> Result<RecipeDto, CreateError>;
    async fn get_recipe(&self, id: Uuid) -> Result<RecipeDto, GetError>;
    async fn list_recipes(&self, list_params: ListParamsDto) -> Result<RecipesListDto, ListError>;
    async fn update_recipe(&self, id: Uuid, request: UpdateDto) -> Result<RecipeDto, UpdateError>;
    async fn delete_recipe(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_recipe(&self, request: CreateDto) -> Result<RecipeDto, CreateError> {
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
    async fn get_recipe(&self, id: Uuid) -> Result<RecipeDto, GetError> {
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
    async fn list_recipes(&self, list_params: ListParamsDto) -> Result<RecipesListDto, ListError> {
        let mut entity = match list_params.name {
            Some(value) => Entity::find().filter(Column::Name.contains(value)),
            None => Entity::find(),
        };
        entity = match list_params.cooking_time_mins {
            Some(value) => entity.filter(Column::CookingTimeMins.lte(value)),
            None => entity,
        };
        Ok(RecipesListDto {
            items: entity
                .order_by_desc(Column::Id)
                .all(&self.database_connection)
                .await
                .map_err(|err| ListError::Unexpected { error: err.into() })?
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
    async fn update_recipe(&self, id: Uuid, request: UpdateDto) -> Result<RecipeDto, UpdateError> {
        let recipe: Model = Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map_err(|err| UpdateError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(UpdateError::NotFound { id })?
            .into();
        let mut recipe: ActiveModel = recipe.into();
        if let Some(name) = request.name {
            recipe.name = Set(name);
        }
        recipe.cooking_time_mins = Set(request.cooking_time_mins);
        recipe.link = Set(request.link);
        recipe.instructions = Set(request.instructions);
        recipe.updated_at = Set(Utc::now().naive_utc());

        Ok(Entity::update(recipe)
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
    async fn delete_recipe(&self, id: Uuid) -> Result<(), DeleteError> {
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
