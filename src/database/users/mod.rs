pub mod dto;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

use self::dto::{CreateDto, UpdateDto, UserDto, UsersListDto};
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
    async fn list_users(&self, list_params: ListParamsDto) -> Result<UsersListDto, ListError>;
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
                if let DbErr::RecordNotInserted = err {
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
    async fn list_users(&self, list_params: ListParamsDto) -> Result<UsersListDto, ListError> {
        Ok(UsersListDto {
            items: match list_params.name {
                Some(value) => Entity::find().filter(Column::Name.eq(value)),
                None => Entity::find(),
            }
            .order_by_asc(Column::Name)
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })?
            .into_iter()
            .map(Into::into)
            .collect(),
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
