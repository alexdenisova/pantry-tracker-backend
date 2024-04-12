use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use db_entities::recipe_users::Model;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub recipe_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListParamsDto {
    pub user_id: Option<Uuid>,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct RecipeUserDto {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct RecipeUsersListDto {
    pub items: Vec<RecipeUserDto>,
}

impl From<CreateDto> for Model {
    fn from(value: CreateDto) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            id: Uuid::new_v4(),
            recipe_id: value.recipe_id,
            user_id: value.user_id,
            created_at: now,
        }
    }
}

impl From<Model> for RecipeUserDto {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            recipe_id: value.recipe_id,
            user_id: value.user_id,
            created_at: value.created_at,
        }
    }
}
