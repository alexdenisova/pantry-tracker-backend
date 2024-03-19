use async_trait::async_trait;
use chrono::NaiveDate;
use entities::pantry_items::{ActiveModel, Column, Entity, Model, Relation};
use sea_orm::{
    sea_query::{Alias, Expr},
    *,
};
use uuid::Uuid;

use crate::{
    errors::{CreateError, DeleteError, GetError, ListError, UpdateError},
    DBClient,
};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_pantry_item(&self, request: Model) -> Result<Model, CreateError>;
    async fn get_pantry_item(&self, id: Uuid) -> Result<Model, GetError>;
    async fn list_pantry_items(&self) -> Result<Vec<Model>, ListError>;
    async fn update_pantry_item(
        &self,
        id: Uuid,
        request: ActiveModel,
    ) -> Result<Model, UpdateError>;
    async fn delete_pantry_item(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_pantry_item(&self, request: Model) -> Result<Model, CreateError> {
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
    async fn get_pantry_item(&self, id: Uuid) -> Result<Model, GetError> {
        Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map_err(|err| GetError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(GetError::NotFound { id })
    }
    async fn list_pantry_items(&self) -> Result<Vec<Model>, ListError> {
        Entity::find()
            .order_by_desc(Column::UpdatedAt)
            .order_by_desc(Column::Id)
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })
    }
    async fn update_pantry_item(
        &self,
        id: Uuid,
        request: ActiveModel,
    ) -> Result<Model, UpdateError> {
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

#[derive(FromQueryResult, Debug)]
pub struct PantryItemsResponse {
    pub id: Uuid,
    pub ingredient_id: Uuid,
    pub ingredient_name: String,
    pub purchase_date: Option<NaiveDate>,
    pub expiration_date: NaiveDate,
    pub quantity: i32,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
}

#[async_trait]
pub trait DatabaseExtra {
    async fn get_pantry_items_of_user(&self, recipe_id: Uuid) -> Result<Vec<Model>, ListError>;
    async fn get_pantry_item_names_of_user(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<PantryItemsResponse>, ListError>;
}

#[async_trait]
impl DatabaseExtra for DBClient {
    async fn get_pantry_items_of_user(&self, user_id: Uuid) -> Result<Vec<Model>, ListError> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })
    }
    async fn get_pantry_item_names_of_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<PantryItemsResponse>, ListError> {
        Entity::find()
            .select_only()
            .columns([
                Column::Id,
                Column::IngredientId,
                Column::PurchaseDate,
                Column::ExpirationDate,
                Column::Quantity,
                Column::WeightGrams,
                Column::VolumeMilliLitres,
            ])
            .column_as(
                Expr::col((
                    Alias::new("ingredients"),
                    entities::ingredients::Column::Name,
                )),
                "ingredient_name",
            )
            .join(JoinType::InnerJoin, Relation::Ingredients.def())
            .filter(Column::UserId.eq(user_id))
            .into_model::<PantryItemsResponse>()
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })
    }
}
