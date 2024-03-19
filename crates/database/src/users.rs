use async_trait::async_trait;
use entities::users::{ActiveModel, Column, Entity, Model};
use sea_orm::*;
use uuid::Uuid;

use crate::{
    errors::{CreateError, DeleteError, GetError, ListError, UpdateError},
    DBClient,
};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_user(&self, request: Model) -> Result<Model, CreateError>;
    async fn get_user(&self, id: Uuid) -> Result<Model, GetError>;
    async fn list_users(&self, predicate: Option<String>) -> Result<Vec<Model>, ListError>;
    async fn update_user(&self, id: Uuid, request: ActiveModel) -> Result<Model, UpdateError>;
    async fn delete_user(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_user(&self, request: Model) -> Result<Model, CreateError> {
        let id = request.id;
        let active_model: ActiveModel = request.into();
        active_model
            .insert(&self.database_connection)
            .await
            .map_err(|err| {
                if let DbErr::RecordNotInserted = err {
                    CreateError::AlreadyExist { id }
                } else {
                    CreateError::Unexpected { error: err.into() }
                }
            })
    }
    async fn get_user(&self, id: Uuid) -> Result<Model, GetError> {
        Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map_err(|err| GetError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(GetError::NotFound { id })
    }
    async fn list_users(&self, predicate: Option<String>) -> Result<Vec<Model>, ListError> {
        match predicate {
            Some(value) => Entity::find().filter(Column::Name.contains(value)),
            None => Entity::find(),
        }
        .order_by_desc(Column::Name)
        .order_by_desc(Column::Id)
        .all(&self.database_connection)
        .await
        .map_err(|err| ListError::Unexpected { error: err.into() })
    }
    async fn update_user(&self, id: Uuid, request: ActiveModel) -> Result<Model, UpdateError> {
        Entity::update(request)
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
            })
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
