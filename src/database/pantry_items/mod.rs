use async_trait::async_trait;
use chrono::Utc;
use db_entities::pantry_items::{ActiveModel, Column, Entity, Model};
use sea_orm::*;
use uuid::Uuid;

pub mod dto;

use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError, UpdateError},
    DBClient,
};

use self::dto::{CreateDto, ListParamsDto, PantryItemDto, PantryItemsListDto, UpdateDto};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_pantry_item(&self, request: CreateDto) -> Result<PantryItemDto, CreateError>;
    async fn get_pantry_item(&self, id: Uuid) -> Result<PantryItemDto, GetError>;
    async fn list_pantry_items(
        &self,
        list_params: ListParamsDto,
    ) -> Result<PantryItemsListDto, ListError>;
    async fn update_pantry_item(
        &self,
        id: Uuid,
        request: UpdateDto,
    ) -> Result<PantryItemDto, UpdateError>;
    async fn delete_pantry_item(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_pantry_item(&self, request: CreateDto) -> Result<PantryItemDto, CreateError> {
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
    async fn get_pantry_item(&self, id: Uuid) -> Result<PantryItemDto, GetError> {
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
    async fn list_pantry_items(
        &self,
        list_params: ListParamsDto,
    ) -> Result<PantryItemsListDto, ListError> {
        Ok(PantryItemsListDto {
            items: match list_params.max_expiration_date {
                Some(value) => Entity::find().filter(Column::ExpirationDate.lte(value)),
                None => Entity::find(),
            }
            .order_by_asc(Column::ExpirationDate)
            .order_by_desc(Column::Id)
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })?
            .into_iter()
            .map(Into::into)
            .collect(),
        })
    }
    async fn update_pantry_item(
        &self,
        id: Uuid,
        request: UpdateDto,
    ) -> Result<PantryItemDto, UpdateError> {
        let pantry_item: Model = Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map_err(|err| UpdateError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(UpdateError::NotFound { id })?
            .into();
        let mut pantry_item: ActiveModel = pantry_item.into();
        pantry_item.purchase_date = Set(request.purchase_date);
        if let Some(date) = request.expiration_date {
            pantry_item.expiration_date = Set(date);
        }
        if let Some(quantity) = request.quantity {
            pantry_item.quantity = Set(quantity);
        }
        pantry_item.weight_grams = Set(request.weight_grams);
        pantry_item.weight_grams = Set(request.volume_milli_litres);
        pantry_item.updated_at = Set(Utc::now().naive_utc());

        Ok(Entity::update(pantry_item)
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
    async fn delete_pantry_item(&self, id: Uuid) -> Result<(), DeleteError> {
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
