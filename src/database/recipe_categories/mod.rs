pub mod dto;

use async_trait::async_trait;
use migrations::{Expr, Func};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoSimpleExpr, JoinType, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select,
};
use uuid::Uuid;

use self::dto::{CreateDto, ListParamsDto, RecipeCategoryDto, RecipeCategoryListDto};

use crate::database::dto::MetadataDto;
use crate::database::errors::{error_code, UNIQUE_VIOLATION_CODE};
use crate::database::recipe_categories::dto::RecipeCategoryJoinDto;
use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError},
    DBClient,
};
use db_entities::recipe_categories::{ActiveModel, Column, Entity, Model};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_recipe_category(
        &self,
        request: CreateDto,
    ) -> Result<RecipeCategoryDto, CreateError>;
    async fn get_recipe_category(&self, id: Uuid) -> Result<RecipeCategoryJoinDto, GetError>;
    async fn list_recipe_categories(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<RecipeCategoryListDto, ListError>;
    async fn get_recipe_categories_metadata(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<MetadataDto, ListError>;
    async fn delete_recipe_category(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_recipe_category(
        &self,
        request: CreateDto,
    ) -> Result<RecipeCategoryDto, CreateError> {
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
    async fn get_recipe_category(&self, id: Uuid) -> Result<RecipeCategoryJoinDto, GetError> {
        join(Entity::find_by_id(id))
            .into_model::<RecipeCategoryJoinDto>()
            .one(&self.database_connection)
            .await
            .map_err(|err| GetError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(GetError::NotFound { id })
    }
    async fn list_recipe_categories(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<RecipeCategoryListDto, ListError> {
        Ok(RecipeCategoryListDto {
            items: list_entity(list_params)
                .limit(list_params.limit)
                .offset(list_params.offset)
                .order_by_asc(db_entities::categories::Column::Name)
                .into_model::<RecipeCategoryJoinDto>()
                .all(&self.database_connection)
                .await
                .map_err(|err| ListError::Unexpected { error: err.into() })?,
        })
    }
    async fn get_recipe_categories_metadata(
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
    async fn delete_recipe_category(&self, id: Uuid) -> Result<(), DeleteError> {
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

fn join(entity: Select<Entity>) -> Select<Entity> {
    entity
        .join(
            JoinType::InnerJoin,
            db_entities::recipe_categories::Relation::Recipes.def(),
        )
        .join(
            JoinType::InnerJoin,
            db_entities::recipe_categories::Relation::Categories.def(),
        )
        .column_as(db_entities::categories::Column::Name, "category_name")
        .column_as(db_entities::recipes::Column::UserId, "user_id")
}

fn list_entity(list_params: &ListParamsDto) -> Select<Entity> {
    let mut entity = match list_params.recipe_id {
        Some(value) => Entity::find().filter(Column::RecipeId.eq(value)),
        None => Entity::find(),
    };
    entity = join(entity);
    if let Some(value) = &list_params.name_contains {
        entity = entity.filter(
            Expr::expr(Func::lower(
                db_entities::categories::Column::Name.into_simple_expr(),
            ))
            .like(format!("%{}%", value.to_lowercase())),
        );
    }
    if let Some(value) = list_params.category_id {
        entity = entity.filter(Column::CategoryId.eq(value));
    }
    if let Some(user_id) = list_params.user_id {
        entity = entity.filter(db_entities::recipes::Column::UserId.eq(user_id));
    }
    entity
}
