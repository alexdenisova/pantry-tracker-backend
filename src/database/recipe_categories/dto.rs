use chrono::{NaiveDateTime, Utc};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use db_entities::recipe_categories::Model;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub recipe_id: Uuid,
    pub category_id: Uuid,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListParamsDto {
    pub recipe_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Serialize, Debug, Clone)]
pub struct RecipeCategoryDto {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub category_id: Uuid,
    pub created_at: NaiveDateTime,
}

impl From<CreateDto> for Model {
    fn from(value: CreateDto) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            id: Uuid::new_v4(),
            recipe_id: value.recipe_id,
            category_id: value.category_id,
            created_at: now,
        }
    }
}

impl From<Model> for RecipeCategoryDto {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            recipe_id: value.recipe_id,
            category_id: value.category_id,
            created_at: value.created_at,
        }
    }
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq, FromQueryResult)]
pub struct RecipeCategoryJoinDto {
    pub id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub recipe_id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Debug)]
pub struct RecipeCategoryListDto {
    pub items: Vec<RecipeCategoryJoinDto>,
}
