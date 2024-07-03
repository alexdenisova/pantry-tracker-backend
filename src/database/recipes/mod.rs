pub mod dto;

use std::collections::HashSet;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Set,
};
use uuid::Uuid;

use self::dto::{CreateDto, ListParamsDto, RecipeDto, RecipesListDto, UpdateDto};
use crate::database::recipes::dto::ListRecipeJoinParamsDto;
use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError, UpdateError},
    DBClient,
};
use db_entities::recipes::{ActiveModel, Column, Entity, Model};
use migrations::{Expr, Func};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_recipe(&self, request: CreateDto) -> Result<RecipeDto, CreateError>;
    async fn get_recipe(&self, id: Uuid) -> Result<RecipeDto, GetError>;
    async fn list_recipes(&self, list_params: ListParamsDto) -> Result<RecipesListDto, ListError>;
    async fn list_recipes_join(
        &self,
        list_params: ListRecipeJoinParamsDto,
    ) -> Result<Vec<RecipeDto>, ListError>;
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
        let mut entity = match list_params.name_contains {
            Some(value) => Entity::find().filter(
                Expr::expr(Func::lower(Expr::col(Column::Name)))
                    .like(format!("%{}%", value.to_lowercase())),
            ),
            None => Entity::find(),
        };
        entity = match list_params.total_time_mins {
            Some(value) => entity.filter(Column::TotalTimeMins.lte(value)),
            None => entity,
        };
        entity = match list_params.user_id {
            Some(value) => entity.filter(Column::UserId.eq(value)),
            None => entity,
        };
        Ok(RecipesListDto {
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
    async fn list_recipes_join(
        &self,
        list_params: ListRecipeJoinParamsDto,
    ) -> Result<Vec<RecipeDto>, ListError> {
        Ok(Entity::find()
            .join(
                JoinType::InnerJoin,
                db_entities::recipes::Relation::RecipeIngredients.def(),
            )
            .filter(Column::UserId.eq(list_params.user_id))
            .filter(
                db_entities::recipe_ingredients::Column::IngredientId
                    .is_in(list_params.ingredient_ids),
            )
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })?
            .into_iter()
            .map(Into::<RecipeDto>::into)
            .collect::<HashSet<RecipeDto>>()
            .into_iter()
            .collect())
    }
    async fn update_recipe(&self, id: Uuid, request: UpdateDto) -> Result<RecipeDto, UpdateError> {
        let recipe: Model = Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map_err(|err| UpdateError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(UpdateError::NotFound { id })?;
        let mut recipe: ActiveModel = recipe.into();
        recipe.user_id = Set(request.user_id);
        recipe.name = Set(request.name);
        recipe.prep_time_mins = Set(request.total_time_mins);
        recipe.total_time_mins = Set(request.total_time_mins);
        recipe.link = Set(request.link);
        recipe.instructions = Set(request.instructions);
        recipe.last_cooked = Set(request.last_cooked);
        recipe.rating = Set(request.rating);
        recipe.notes = Set(request.notes);
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
