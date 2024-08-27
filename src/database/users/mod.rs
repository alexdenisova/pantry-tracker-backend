pub mod dto;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Select,
};
use uuid::Uuid;

use self::dto::{CreateDto, UpdateDto, UserDto, UsersListDto};
use crate::database::dto::MetadataDto;
use crate::database::errors::{error_code, UNIQUE_VIOLATION_CODE};
use crate::database::users::dto::ListParamsDto;
use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError, UpdateError},
    DBClient,
};
use db_entities::users::{ActiveModel, Column, Entity, Model};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_user(&self, request: CreateDto) -> Result<UserDto, CreateError>;
    async fn get_user(&self, id: Uuid) -> Result<UserDto, GetError>;
    async fn list_users(&self, list_params: &ListParamsDto) -> Result<UsersListDto, ListError>;
    async fn get_users_metadata(
        &self,
        list_params: &ListParamsDto,
    ) -> Result<MetadataDto, ListError>;
    async fn update_user(&self, id: Uuid, request: UpdateDto) -> Result<UserDto, UpdateError>;
    async fn delete_user(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_user(&self, request: CreateDto) -> Result<UserDto, CreateError> {
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
    async fn get_user(&self, id: Uuid) -> Result<UserDto, GetError> {
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
    async fn list_users(&self, list_params: &ListParamsDto) -> Result<UsersListDto, ListError> {
        Ok(UsersListDto {
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
    async fn get_users_metadata(
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
    async fn update_user(&self, id: Uuid, request: UpdateDto) -> Result<UserDto, UpdateError> {
        let active_model: ActiveModel = ActiveModel {
            id: ActiveValue::Set(id),
            name: ActiveValue::Set(request.name),
            password_hash: ActiveValue::Set(request.password_hash),
            admin: ActiveValue::Set(request.admin.unwrap_or(false)),
            updated_at: ActiveValue::Set(Utc::now().naive_utc()),
            ..Default::default()
        };
        Ok(Entity::update(active_model)
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
    async fn delete_user(&self, id: Uuid) -> Result<(), DeleteError> {
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
    match &list_params.name {
        Some(value) => Entity::find().filter(Column::Name.eq(value)),
        None => Entity::find(),
    }
}
