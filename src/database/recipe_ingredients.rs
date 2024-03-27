use async_trait::async_trait;
use db_entities::recipe_ingredients::{ActiveModel, Column, Entity, Model, Relation};
use sea_orm::{
    sea_query::{Alias, Expr},
    *,
};
use uuid::Uuid;

use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError, UpdateError},
    DBClient,
};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_recipe_ingredient(&self, request: Model) -> Result<Model, CreateError>;
    async fn get_recipe_ingredient(&self, id: Uuid) -> Result<Model, GetError>;
    async fn list_recipe_ingredients(&self) -> Result<Vec<Model>, ListError>;
    async fn update_recipe_ingredient(
        &self,
        id: Uuid,
        request: ActiveModel,
    ) -> Result<Model, UpdateError>;
    async fn delete_recipe_ingredient(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_recipe_ingredient(&self, request: Model) -> Result<Model, CreateError> {
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
    async fn get_recipe_ingredient(&self, id: Uuid) -> Result<Model, GetError> {
        Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map_err(|err| GetError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(GetError::NotFound { id })
    }
    async fn list_recipe_ingredients(&self) -> Result<Vec<Model>, ListError> {
        Entity::find()
            .order_by_desc(Column::UpdatedAt)
            .order_by_desc(Column::Id)
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })
    }
    async fn update_recipe_ingredient(
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
    async fn delete_recipe_ingredient(&self, id: Uuid) -> Result<(), DeleteError> {
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
    async fn get_all_ingredients_of_recipe(&self, recipe_id: Uuid)
        -> Result<Vec<Model>, ListError>;
    async fn get_ingredient_names_of_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<RecipeIngredientsResponse>, ListError>;
}

#[async_trait]
impl DatabaseExtra for DBClient {
    async fn get_all_ingredients_of_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<Model>, ListError> {
        Entity::find()
            .filter(Column::RecipeId.eq(recipe_id))
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })
    }
    async fn get_ingredient_names_of_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<RecipeIngredientsResponse>, ListError> {
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
                    db_entities::ingredients::Column::Name,
                )),
                "ingredient_name",
            )
            .join(JoinType::InnerJoin, Relation::Ingredients.def())
            .filter(Column::RecipeId.eq(recipe_id))
            .into_model::<RecipeIngredientsResponse>()
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })
        //  select id, amount, unit, optional, ingredients.name as ingredient_name
        //  from recipe_ingredients
        //  inner join ingredients on recipe_ingredients.ingredient_id = ingredients.id
        //  where recipes.id = '$recipe_id'
    }
}
