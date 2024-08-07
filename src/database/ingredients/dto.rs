use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use db_entities::ingredients::Model;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub name: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListParamsDto {
    pub name: Option<String>,
    pub name_contains: Option<String>,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct IngredientDto {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct IngredientsListDto {
    pub items: Vec<IngredientDto>,
}

impl From<CreateDto> for Model {
    fn from(value: CreateDto) -> Self {
        let now = Utc::now().naive_utc();

        Self {
            id: Uuid::new_v4(),
            name: value.name,
            created_at: now,
        }
    }
}

impl From<Model> for IngredientDto {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            created_at: value.created_at,
        }
    }
}
