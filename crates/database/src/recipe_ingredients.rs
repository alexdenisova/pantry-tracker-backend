use async_trait::async_trait;
use chrono::NaiveDateTime;
use entities::recipe_ingredients::{ActiveModel, Column, Entity, Model, Relation};
use sea_orm::{
    sea_query::{Alias, Expr},
    *,
};
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

#[derive(FromQueryResult, Debug)]
pub struct RecipeIngredientsResponse {
    pub id: Uuid,
    pub ingredient_id: Uuid,
    pub ingredient_name: String,
    pub amount: i32,
    pub unit: String,
    pub optional: bool,
}

#[async_trait]
pub trait DatabaseExtra {
    async fn get_all_ingredients_of_recipe(&self, recipe_id: Uuid) -> Result<Vec<Response>, DbErr>;
    async fn get_ingredient_names_of_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<RecipeIngredientsResponse>, DbErr>;
}

#[async_trait]
impl DatabaseExtra for DBClient {
    async fn get_all_ingredients_of_recipe(&self, recipe_id: Uuid) -> Result<Vec<Response>, DbErr> {
        Entity::find()
            .filter(Column::RecipeId.eq(recipe_id))
            .all(&self.database_connection)
            .await
            .map(|x| x.into_iter().map(Into::into).collect())
    }
    async fn get_ingredient_names_of_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<RecipeIngredientsResponse>, DbErr> {
        Entity::find()
            .select_only()
            .columns([
                Column::Id,
                Column::IngredientId,
                Column::Amount,
                Column::Unit,
                Column::Optional,
            ])
            .column_as(
                Expr::col((
                    Alias::new("ingredients"),
                    entities::ingredients::Column::Name,
                )),
                "ingredient_name",
            )
            .join(JoinType::InnerJoin, Relation::Ingredients.def())
            .filter(Column::RecipeId.eq(recipe_id))
            .into_model::<RecipeIngredientsResponse>()
            .all(&self.database_connection)
            .await
            .map(|x| x.into_iter().map(Into::into).collect())
        //  select id, amount, unit, optional, ingredients.name as ingredient_name
        //  from recipe_ingredients
        //  inner join ingredients on recipe_ingredients.ingredient_id = ingredients.id
        //  where recipes.id = '$recipe_id'
    }
}
