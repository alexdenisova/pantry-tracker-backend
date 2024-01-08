use async_trait::async_trait;
use chrono::NaiveDateTime;
use entities::recipe_ingredients::{ActiveModel, Column, Entity, Model};
use sea_orm::*;
use uuid::Uuid;

use crate::DBClient;

pub struct Request {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
    pub amount: i32,
    pub unit: String,
    pub optional: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct Response {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
    pub amount: i32,
    pub unit: String,
    pub optional: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Request> for Model {
    // TODO: make this a macro
    fn from(value: Request) -> Self {
        Model {
            id: value.id,
            recipe_id: value.recipe_id,
            ingredient_id: value.ingredient_id,
            amount: value.amount,
            unit: value.unit,
            optional: value.optional,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<Model> for Response {
    fn from(value: Model) -> Self {
        Response {
            id: value.id,
            recipe_id: value.recipe_id,
            ingredient_id: value.ingredient_id,
            amount: value.amount,
            unit: value.unit,
            optional: value.optional,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_recipe_ingredient(&self, request: Request) -> Result<Response, DbErr>;
    async fn get_recipe_ingredient(&self, id: Uuid) -> Result<Option<Response>, DbErr>;
    async fn list_recipe_ingredients(&self) -> Result<Vec<Response>, DbErr>;
    async fn update_recipe_ingredient(&self, id: Uuid, request: Request)
        -> Result<Response, DbErr>;
    async fn delete_recipe_ingredient(&self, id: Uuid) -> Result<(), DbErr>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_recipe_ingredient(&self, request: Request) -> Result<Response, DbErr> {
        let model: Model = request.into();
        let active_model: ActiveModel = model.into();
        active_model
            .insert(&self.database_connection)
            .await
            .map(Into::into)
    }
    async fn get_recipe_ingredient(&self, id: Uuid) -> Result<Option<Response>, DbErr> {
        Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map(|x| x.map(Into::into))
    }
    async fn list_recipe_ingredients(&self) -> Result<Vec<Response>, DbErr> {
        Entity::find()
            .order_by_desc(Column::UpdatedAt)
            .order_by_desc(Column::Id)
            .all(&self.database_connection)
            .await
            .map(|x| x.into_iter().map(Into::into).collect())
    }
    async fn update_recipe_ingredient(
        &self,
        id: Uuid,
        request: Request,
    ) -> Result<Response, DbErr> {
        let model: Model = request.into();
        let active_model: ActiveModel = model.into();

        Entity::update(active_model)
            .filter(Column::Id.eq(id))
            .exec(&self.database_connection)
            .await
            .map(Into::into)
    }
    async fn delete_recipe_ingredient(&self, id: Uuid) -> Result<(), DbErr> {
        Entity::delete_by_id(id)
            .exec(&self.database_connection)
            .await
            .map(|_| ())
    }
}
