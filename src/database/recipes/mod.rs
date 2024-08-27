pub mod dto;

use std::collections::HashSet;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, JoinType, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait, Select, Set,
};
use uuid::Uuid;

use self::dto::{CreateDto, ListParamsDto, RecipeDto, RecipesListDto, UpdateDto};
use crate::database::dto::MetadataDto;
use crate::database::errors::{error_code, UNIQUE_VIOLATION_CODE};
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
    async fn list_recipes(&self, list_params: &ListParamsDto) -> Result<RecipesListDto, ListError>;
    async fn get_recipes_metadata(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<MetadataDto, ListError>;
    async fn list_recipes_join(
        &self,
        list_params: &ListRecipeJoinParamsDto,
    ) -> Result<Vec<RecipeDto>, ListError>;
    async fn get_recipes_join_metadata(
        &self,
        list_params: &ListRecipeJoinParamsDto,
    ) -> Result<MetadataDto, ListError>;
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
                if error_code(&err) == Some(UNIQUE_VIOLATION_CODE.to_owned()) {
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
    async fn list_recipes(&self, list_params: &ListParamsDto) -> Result<RecipesListDto, ListError> {
        Ok(RecipesListDto {
            items: list_entity(list_params)
                .limit(list_params.limit)
                .offset(list_params.offset)
                .order_by_desc(Column::UpdatedAt)
                .all(&self.database_connection)
                .await
                .map_err(|err| ListError::Unexpected { error: err.into() })?
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
    async fn get_recipes_metadata(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<MetadataDto, ListError> {
        let total_count = list_entity(list_params)
            .count(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })?;
        Ok(MetadataDto {
            page: list_params.offset / list_params.limit + 1,
            per_page: list_params.limit,
            page_count: total_count / list_params.limit + 1,
            total_count,
        })
    }
    async fn list_recipes_join(
        &self,
        list_params: &ListRecipeJoinParamsDto,
    ) -> Result<Vec<RecipeDto>, ListError> {
        Ok(list_join_entity(list_params)
            .limit(list_params.limit)
            .offset(list_params.offset)
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })?
            .into_iter()
            .map(Into::<RecipeDto>::into)
            .collect::<HashSet<RecipeDto>>()
            .into_iter()
            .collect())
    }
    async fn get_recipes_join_metadata(
        &self,
        list_params: &ListRecipeJoinParamsDto,
    ) -> Result<MetadataDto, ListError> {
        let total_count = list_join_entity(list_params)
            .count(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })?;
        Ok(MetadataDto {
            page: list_params.offset / list_params.limit + 1,
            per_page: list_params.limit,
            page_count: total_count / list_params.limit + 1,
            total_count,
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

fn list_entity(list_params: &ListParamsDto) -> Select<Entity> {
    let mut entity = Entity::find();
    if let Some(value) = &list_params.name_contains {
        entity = entity.filter(
            Expr::expr(Func::lower(Expr::col(Column::Name)))
                .like(format!("%{}%", value.to_lowercase())),
        );
    }
    if let Some(value) = list_params.total_time_mins {
        entity = entity.filter(Column::TotalTimeMins.lte(value));
    }
    if let Some(value) = list_params.user_id {
        entity = entity.filter(Column::UserId.eq(value));
    }
    entity
}

fn list_join_entity(list_params: &ListRecipeJoinParamsDto) -> Select<Entity> {
    Entity::find()
        .join(
            JoinType::InnerJoin,
            db_entities::recipes::Relation::RecipeIngredients.def(),
        )
        .filter(Column::UserId.eq(list_params.user_id))
        .filter(
            db_entities::recipe_ingredients::Column::IngredientId
                .is_in(list_params.ingredient_ids.clone()),
        )
}
