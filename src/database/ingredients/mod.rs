use async_trait::async_trait;
use db_entities::ingredients::{ActiveModel, Column, Entity, Model};
use sea_orm::*;
use uuid::Uuid;

pub mod dto;

use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError, UpdateError},
    DBClient,
};

use self::dto::{CreateDto, IngredientDto, IngredientsListDto, ListParamsDto, UpdateDto};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_ingredient(&self, request: CreateDto) -> Result<IngredientDto, CreateError>;
    async fn get_ingredient(&self, id: Uuid) -> Result<IngredientDto, GetError>;
    async fn list_ingredients(
        &self,
        list_params: ListParamsDto,
    ) -> Result<IngredientsListDto, ListError>;
    async fn update_ingredient(
        &self,
        id: Uuid,
        request: UpdateDto,
    ) -> Result<IngredientDto, UpdateError>;
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
                if let DbErr::RecordNotInserted = err {
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
        list_params: ListParamsDto,
    ) -> Result<IngredientsListDto, ListError> {
        Ok(IngredientsListDto {
            items: match list_params.predicate {
                Some(value) => Entity::find().filter(Column::Name.contains(value)),
                None => Entity::find(),
            }
            .order_by_desc(Column::Name)
            .order_by_desc(Column::Id)
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })?
            .into_iter()
            .map(Into::into)
            .collect(),
        })
    }
    async fn update_ingredient(
        &self,
        id: Uuid,
        request: UpdateDto,
    ) -> Result<IngredientDto, UpdateError> {
        let active_model: ActiveModel = ActiveModel {
            id: ActiveValue::Set(id),
            name: ActiveValue::Set(request.name),
            can_be_eaten_raw: ActiveValue::Set(request.can_be_eaten_raw),
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
