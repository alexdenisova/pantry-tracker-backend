use async_trait::async_trait;
use entities::recipe_users::{ActiveModel, Column, Entity, Model, Relation};
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
    async fn create_recipe_user(&self, request: Model) -> Result<Model, CreateError>;
    async fn get_recipe_user(&self, id: Uuid) -> Result<Model, GetError>;
    async fn list_recipe_users(&self) -> Result<Vec<Model>, ListError>;
    async fn update_recipe_user(
        &self,
        id: Uuid,
        request: ActiveModel,
    ) -> Result<Model, UpdateError>;
    async fn delete_recipe_user(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_recipe_user(&self, request: Model) -> Result<Model, CreateError> {
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
    async fn get_recipe_user(&self, id: Uuid) -> Result<Model, GetError> {
        Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map_err(|err| GetError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(GetError::NotFound { id })
    }
    async fn list_recipe_users(&self) -> Result<Vec<Model>, ListError> {
        Entity::find()
            .order_by_desc(Column::CreatedAt)
            .order_by_desc(Column::Id)
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })
    }
    async fn update_recipe_user(
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
    async fn delete_recipe_user(&self, id: Uuid) -> Result<(), DeleteError> {
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
    async fn get_recipes_of_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RecipeUsersResponse>, ListError>;
}

#[async_trait]
impl DatabaseExtra for DBClient {
    async fn get_recipes_of_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RecipeUsersResponse>, ListError> {
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
            .map_err(|err| ListError::Unexpected { error: err.into() })
    }
}
