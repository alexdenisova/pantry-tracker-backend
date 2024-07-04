pub mod dto;

use async_trait::async_trait;
use chrono::Utc;
use migrations::{Expr, Func};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Set,
};
use uuid::Uuid;

use self::dto::{CreateDto, ListParamsDto, PantryItemDto, PantryItemsListDto, UpdateDto};
use crate::database::pantry_items::dto::PantryItemJoinDto;
use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError, UpdateError},
    DBClient,
};
use db_entities::pantry_items::{ActiveModel, Column, Entity, Model};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_pantry_item(&self, request: CreateDto) -> Result<PantryItemDto, CreateError>;
    async fn get_pantry_item(&self, id: Uuid) -> Result<PantryItemDto, GetError>;
    async fn get_pantry_item_join(&self, id: Uuid) -> Result<PantryItemJoinDto, GetError>;
    async fn list_pantry_items_join(
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
    async fn get_pantry_item_join(&self, id: Uuid) -> Result<PantryItemJoinDto, GetError> {
        Ok(Entity::find_by_id(id)
            .join(
                JoinType::InnerJoin,
                db_entities::pantry_items::Relation::Ingredients.def(),
            )
            .column_as(db_entities::ingredients::Column::Name, "ingredient_name")
            .into_model::<PantryItemJoinDto>()
            .one(&self.database_connection)
            .await
            .map_err(|err| GetError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(GetError::NotFound { id })?)
    }
    async fn list_pantry_items_join(
        &self,
        list_params: ListParamsDto,
    ) -> Result<PantryItemsListDto, ListError> {
        let mut entity = match list_params.user_id {
            Some(value) => Entity::find().filter(Column::UserId.eq(value)),
            None => Entity::find(),
        };
        entity = match list_params.ingredient_id {
            Some(value) => entity.filter(Column::IngredientId.eq(value)),
            None => entity,
        };
        if let Some(value) = list_params.name_contains {
            entity = entity.filter(
                Expr::expr(Func::lower(Expr::col(
                    db_entities::ingredients::Column::Name,
                )))
                .like(format!("%{}%", value.to_lowercase())),
            );
        };
        Ok(PantryItemsListDto {
            items: match list_params.max_expiration_date {
                Some(value) => entity.filter(Column::ExpirationDate.lte(value)),
                None => entity,
            }
            .join(
                JoinType::InnerJoin,
                db_entities::pantry_items::Relation::Ingredients.def(),
            )
            .column_as(db_entities::ingredients::Column::Name, "ingredient_name")
            .order_by_desc(Column::UpdatedAt)
            .into_model::<PantryItemJoinDto>()
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })?,
        })
    }
    async fn update_pantry_item(
        &self,
        id: Uuid,
        request: UpdateDto,
    ) -> Result<PantryItemDto, UpdateError> {
        println!("in 0");
        let pantry_item: Model = Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map_err(|err| UpdateError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(UpdateError::NotFound { id })?;
        println!("in");
        let mut pantry_item: ActiveModel = pantry_item.into();
        pantry_item.ingredient_id = Set(request.ingredient_id);
        pantry_item.user_id = Set(request.user_id);
        pantry_item.purchase_date = Set(request.purchase_date);
        pantry_item.expiration_date = Set(request.expiration_date);
        pantry_item.quantity = Set(request.quantity);
        pantry_item.weight_grams = Set(request.weight_grams);
        pantry_item.volume_milli_litres = Set(request.volume_milli_litres);
        pantry_item.essential = Set(request.essential);
        pantry_item.running_low = Set(request.running_low);
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
