pub mod dto;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoSimpleExpr, JoinType, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select, Set,
};
use uuid::Uuid;

use self::dto::{
    CreateDto, ListParamsDto, RecipeIngredientDto, RecipeIngredientsListDto, UpdateDto,
};

use crate::database::dto::MetadataDto;
use crate::database::errors::{error_code, UNIQUE_VIOLATION_CODE};
use crate::database::recipe_ingredients::dto::RecipeIngredientJoinDto;
use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError, UpdateError},
    DBClient,
};
use db_entities::recipe_ingredients::{ActiveModel, Column, Entity, Model};
use migrations::{Expr, Func};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_recipe_ingredient(
        &self,
        request: CreateDto,
    ) -> Result<RecipeIngredientDto, CreateError>;
    async fn get_recipe_ingredient(&self, id: Uuid) -> Result<RecipeIngredientDto, GetError>;
    async fn list_recipe_ingredients(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<RecipeIngredientsListDto, ListError>;
    async fn get_recipe_ingredients_metadata(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<MetadataDto, ListError>;
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
                if error_code(&err) == Some(UNIQUE_VIOLATION_CODE.to_owned()) {
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
        list_params: &ListParamsDto,
    ) -> Result<RecipeIngredientsListDto, ListError> {
        Ok(RecipeIngredientsListDto {
            items: list_entity(list_params)
                .limit(list_params.limit)
                .offset(list_params.offset)
                .order_by_asc(db_entities::ingredients::Column::Name)
                .into_model::<RecipeIngredientJoinDto>()
                .all(&self.database_connection)
                .await
                .map_err(|err| ListError::Unexpected { error: err.into() })?,
        })
    }
    async fn get_recipe_ingredients_metadata(
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

fn list_entity(list_params: &ListParamsDto) -> Select<Entity> {
    let mut entity = match list_params.recipe_id {
        Some(value) => Entity::find().filter(Column::RecipeId.eq(value)),
        None => Entity::find(),
    }
    .join(
        JoinType::InnerJoin,
        db_entities::recipe_ingredients::Relation::Recipes.def(),
    )
    .join(
        JoinType::InnerJoin,
        db_entities::recipe_ingredients::Relation::Ingredients.def(),
    )
    .column_as(db_entities::ingredients::Column::Name, "ingredient_name")
    .column_as(db_entities::recipes::Column::Name, "recipe_name");
    if let Some(value) = &list_params.name_contains {
        entity = entity.filter(
            Expr::expr(Func::lower(
                db_entities::ingredients::Column::Name.into_simple_expr(),
            ))
            .like(format!("%{}%", value.to_lowercase())),
        );
    }
    if let Some(user_id) = list_params.user_id {
        entity = entity
            .filter(db_entities::recipes::Column::UserId.eq(user_id))
            .distinct_on([(
                db_entities::ingredients::Entity,
                db_entities::ingredients::Column::Name,
            )]);
    }
    entity
}
