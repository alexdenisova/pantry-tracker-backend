use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};
use entities::pantry_items::{ActiveModel, Column, Entity, Model};
use sea_orm::*;
use uuid::Uuid;

use crate::DBClient;

pub struct Request {
    pub id: Uuid,
    pub ingredient_id: Uuid,
    pub user_id: Uuid,
    pub purchase_date: Option<NaiveDate>,
    pub expiration_date: NaiveDate,
    pub quantity: i32,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct Response {
    pub id: Uuid,
    pub ingredient_id: Uuid,
    pub user_id: Uuid,
    pub purchase_date: Option<NaiveDate>,
    pub expiration_date: NaiveDate,
    pub quantity: i32,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Request> for Model {
    // TODO: make this a macro
    fn from(value: Request) -> Self {
        Model {
            id: value.id,
            ingredient_id: value.ingredient_id,
            user_id: value.user_id,
            purchase_date: value.purchase_date,
            expiration_date: value.expiration_date,
            quantity: value.quantity,
            weight_grams: value.weight_grams,
            volume_milli_litres: value.volume_milli_litres,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<Model> for Response {
    fn from(value: Model) -> Self {
        Response {
            id: value.id,
            ingredient_id: value.ingredient_id,
            user_id: value.user_id,
            purchase_date: value.purchase_date,
            expiration_date: value.expiration_date,
            quantity: value.quantity,
            weight_grams: value.weight_grams,
            volume_milli_litres: value.volume_milli_litres,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_pantry_item(&self, request: Request) -> Result<Response, DbErr>;
    async fn get_pantry_item(&self, id: Uuid) -> Result<Option<Response>, DbErr>;
    async fn list_pantry_items(&self) -> Result<Vec<Response>, DbErr>;
    async fn update_pantry_item(&self, id: Uuid, request: Request) -> Result<Response, DbErr>;
    async fn delete_pantry_item(&self, id: Uuid) -> Result<(), DbErr>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_pantry_item(&self, request: Request) -> Result<Response, DbErr> {
        let model: Model = request.into();
        let active_model: ActiveModel = model.into();
        active_model
            .insert(&self.database_connection)
            .await
            .map(Into::into)
    }
    async fn get_pantry_item(&self, id: Uuid) -> Result<Option<Response>, DbErr> {
        Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map(|x| x.map(Into::into))
    }
    async fn list_pantry_items(&self) -> Result<Vec<Response>, DbErr> {
        Entity::find()
        .order_by_desc(Column::UpdatedAt)
        .order_by_desc(Column::Id)
        .all(&self.database_connection)
        .await
        .map(|x| x.into_iter().map(Into::into).collect())
    }
    async fn update_pantry_item(&self, id: Uuid, request: Request) -> Result<Response, DbErr> {
        let model: Model = request.into();
        let active_model: ActiveModel = model.into();

        Entity::update(active_model)
            .filter(Column::Id.eq(id))
            .exec(&self.database_connection)
            .await
            .map(Into::into)
    }
    async fn delete_pantry_item(&self, id: Uuid) -> Result<(), DbErr> {
        Entity::delete_by_id(id)
            .exec(&self.database_connection)
            .await
            .map(|_| ())
    }
}
