pub mod dto;

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Select,
};
use uuid::Uuid;

use self::dto::{CategoryDto, CategoryListDto, CreateDto, ListParamsDto};
use crate::database::dto::MetadataDto;
use crate::database::errors::{error_code, UNIQUE_VIOLATION_CODE};
use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError},
    DBClient,
};
use db_entities::categories::{ActiveModel, Column, Entity, Model};
use migrations::{Expr, Func};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_category(&self, request: CreateDto) -> Result<CategoryDto, CreateError>;
    async fn get_category(&self, id: Uuid) -> Result<CategoryDto, GetError>;
    async fn list_categories(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<CategoryListDto, ListError>;
    async fn get_categories_metadata(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<MetadataDto, ListError>;
    async fn delete_category(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_category(&self, request: CreateDto) -> Result<CategoryDto, CreateError> {
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
    async fn get_category(&self, id: Uuid) -> Result<CategoryDto, GetError> {
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
    async fn list_categories(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<CategoryListDto, ListError> {
        Ok(CategoryListDto {
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
    async fn get_categories_metadata(
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
    async fn delete_category(&self, id: Uuid) -> Result<(), DeleteError> {
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
    if let Some(value) = &list_params.name {
        entity = entity.filter(
            Expr::expr(Func::lower(Expr::col(Column::Name))).eq(value.to_lowercase().to_string()),
        );
    }
    entity
}
