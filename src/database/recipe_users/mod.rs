pub mod dto;

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use self::dto::{CreateDto, ListParamsDto, RecipeUserDto, RecipeUsersListDto};
use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError},
    DBClient,
};
use db_entities::recipe_users::{ActiveModel, Column, Entity, Model};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_recipe_user(&self, request: CreateDto) -> Result<RecipeUserDto, CreateError>;
    async fn get_recipe_user(&self, id: Uuid) -> Result<RecipeUserDto, GetError>;
    async fn list_recipe_users(
        &self,
        list_params: ListParamsDto,
    ) -> Result<RecipeUsersListDto, ListError>;
    async fn delete_recipe_user(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_recipe_user(&self, request: CreateDto) -> Result<RecipeUserDto, CreateError> {
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
    async fn get_recipe_user(&self, id: Uuid) -> Result<RecipeUserDto, GetError> {
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
    async fn list_recipe_users(
        &self,
        list_params: ListParamsDto,
    ) -> Result<RecipeUsersListDto, ListError> {
        Ok(RecipeUsersListDto {
            items: match list_params.user_id {
                Some(value) => Entity::find().filter(Column::RecipeId.eq(value)),
                None => Entity::find(),
            }
            .order_by_desc(Column::CreatedAt)
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })?
            .into_iter()
            .map(Into::into)
            .collect(),
        })
    }
    async fn delete_recipe_user(&self, id: Uuid) -> Result<(), DeleteError> {
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
