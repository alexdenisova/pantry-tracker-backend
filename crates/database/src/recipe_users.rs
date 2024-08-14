use async_trait::async_trait;
use chrono::NaiveDateTime;
use entities::recipe_users::{ActiveModel, Column, Entity, Model, Relation};
use sea_orm::{
    sea_query::{Alias, Expr},
    *,
};
use uuid::Uuid;

use crate::DBClient;

pub struct Request {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
}

pub struct Response {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
}

impl From<Request> for Model {
    // TODO: make this a macro
    fn from(value: Request) -> Self {
        Model {
            id: value.id,
            recipe_id: value.recipe_id,
            user_id: value.user_id,
            created_at: value.created_at,
        }
    }
}

impl From<Model> for Response {
    fn from(value: Model) -> Self {
        Response {
            id: value.id,
            recipe_id: value.recipe_id,
            user_id: value.user_id,
            created_at: value.created_at,
        }
    }
}

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_recipe_user(&self, request: Request) -> Result<Response, DbErr>;
    async fn get_recipe_user(&self, id: Uuid) -> Result<Option<Response>, DbErr>;
    async fn list_recipe_users(&self) -> Result<Vec<Response>, DbErr>;
    async fn update_recipe_user(&self, id: Uuid, request: Request) -> Result<Response, DbErr>;
    async fn delete_recipe_user(&self, id: Uuid) -> Result<(), DbErr>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_recipe_user(&self, request: Request) -> Result<Response, DbErr> {
        let model: Model = request.into();
        let active_model: ActiveModel = model.into();
        active_model
            .insert(&self.database_connection)
            .await
            .map(Into::into)
    }
    async fn get_recipe_user(&self, id: Uuid) -> Result<Option<Response>, DbErr> {
        Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map(|x| x.map(Into::into))
    }
    async fn list_recipe_users(&self) -> Result<Vec<Response>, DbErr> {
        Entity::find()
            .order_by_desc(Column::CreatedAt)
            .order_by_desc(Column::Id)
            .all(&self.database_connection)
            .await
            .map(|x| x.into_iter().map(Into::into).collect())
    }
    async fn update_recipe_user(&self, id: Uuid, request: Request) -> Result<Response, DbErr> {
        let model: Model = request.into();
        let active_model: ActiveModel = model.into();

        Entity::update(active_model)
            .filter(Column::Id.eq(id))
            .exec(&self.database_connection)
            .await
            .map(Into::into)
    }
    async fn delete_recipe_user(&self, id: Uuid) -> Result<(), DbErr> {
        Entity::delete_by_id(id)
            .exec(&self.database_connection)
            .await
            .map(|_| ())
    }
}

#[derive(FromQueryResult, Debug)]
pub struct RecipeUsersResponse {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub recipe_name: String,
    pub cooking_time_mins: Option<i32>,
    pub link: Option<String>,
    pub instructions: Option<String>,
}

#[async_trait]
pub trait DatabaseExtra {
    async fn get_recipes_of_user(&self, user_id: Uuid) -> Result<Vec<RecipeUsersResponse>, DbErr>;
}

#[async_trait]
impl DatabaseExtra for DBClient {
    async fn get_recipes_of_user(&self, user_id: Uuid) -> Result<Vec<RecipeUsersResponse>, DbErr> {
        Entity::find()
            .select_only()
            .columns([Column::Id, Column::RecipeId])
            .columns([
                entities::recipes::Column::CookingTimeMins,
                entities::recipes::Column::Link,
                entities::recipes::Column::Instructions,
            ])
            .column_as(
                Expr::col((Alias::new("recipes"), entities::recipes::Column::Name)),
                "recipe_name",
            )
            .join(JoinType::InnerJoin, Relation::Recipes.def())
            .filter(Column::UserId.eq(user_id))
            .into_model::<RecipeUsersResponse>()
            .all(&self.database_connection)
            .await
            .map(|x| x.into_iter().map(Into::into).collect())
    }
}
