pub mod dto;

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    Select,
};
use uuid::Uuid;

use self::dto::{CreateDto, IngredientDto, IngredientsListDto, ListParamsDto};
use crate::database::dto::MetadataDto;
use crate::database::errors::{error_code, UNIQUE_VIOLATION_CODE};
use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError},
    DBClient,
};
use db_entities::ingredients::{ActiveModel, Column, Entity, Model};
use migrations::{Expr, Func};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_ingredient(&self, request: CreateDto) -> Result<IngredientDto, CreateError>;
    async fn get_ingredient(&self, id: Uuid) -> Result<IngredientDto, GetError>;
    async fn list_ingredients(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<IngredientsListDto, ListError>;
    async fn get_ingredients_metadata(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<MetadataDto, ListError>;
    async fn delete_ingredient(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_ingredient(&self, request: CreateDto) -> Result<IngredientDto, CreateError> {
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
    async fn get_ingredient(&self, id: Uuid) -> Result<IngredientDto, GetError> {
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
    async fn list_ingredients(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<IngredientsListDto, ListError> {
        Ok(IngredientsListDto {
            items: list_entity(list_params)
                .limit(list_params.limit)
                .offset(list_params.offset)
                .order_by_asc(Column::Name)
                .all(&self.database_connection)
                .await
                .map_err(|err| ListError::Unexpected { error: err.into() })?
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
    async fn get_ingredients_metadata(
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
    async fn delete_ingredient(&self, id: Uuid) -> Result<(), DeleteError> {
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
    match &list_params.name {
        Some(value) => {
            entity = entity.filter(
                Expr::expr(Func::lower(Expr::col(Column::Name)))
                    .eq(value.to_lowercase().to_string()),
            );
        }
        None => {
            if let Some(value) = &list_params.name_contains {
                entity = entity.filter(
                    Expr::expr(Func::lower(Expr::col(Column::Name)))
                        .like(format!("%{}%", value.to_lowercase())),
                );
            }
        }
    }
    entity
}
