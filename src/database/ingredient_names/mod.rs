pub mod dto;

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use self::dto::{CreateDto, IngredientNameDto, IngredientNamesListDto, ListParamsDto};
use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError},
    DBClient,
};
use db_entities::ingredient_names::{ActiveModel, Column, Entity, Model};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_ingredient_name(
        &self,
        request: CreateDto,
    ) -> Result<IngredientNameDto, CreateError>;
    async fn get_ingredient_name(&self, id: Uuid) -> Result<IngredientNameDto, GetError>;
    async fn list_ingredient_names(
        &self,
        list_params: ListParamsDto,
    ) -> Result<IngredientNamesListDto, ListError>;
    async fn delete_ingredient_name(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_ingredient_name(
        &self,
        request: CreateDto,
    ) -> Result<IngredientNameDto, CreateError> {
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
    async fn get_ingredient_name(&self, id: Uuid) -> Result<IngredientNameDto, GetError> {
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
    async fn list_ingredient_names(
        &self,
        list_params: ListParamsDto,
    ) -> Result<IngredientNamesListDto, ListError> {
        Ok(IngredientNamesListDto {
            items: match list_params.ingredient_id {
                Some(value) => Entity::find().filter(Column::IngredientId.eq(value)),
                None => Entity::find(),
            }
            .order_by_desc(Column::Name)
            .order_by_desc(Column::CreatedAt)
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })?
            .into_iter()
            .map(Into::into)
            .collect(),
        })
    }
    async fn delete_ingredient_name(&self, id: Uuid) -> Result<(), DeleteError> {
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
