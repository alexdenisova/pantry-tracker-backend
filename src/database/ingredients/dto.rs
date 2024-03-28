use chrono::{NaiveDateTime, Utc};
use db_entities::ingredients::Model;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub name: String,
    pub can_be_eaten_raw: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateDto {
    pub name: Option<String>,
    pub can_be_eaten_raw: Option<bool>,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct IngredientDto {
    pub id: Uuid,
    pub name: String,
    pub can_be_eaten_raw: bool,
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
            can_be_eaten_raw: value.can_be_eaten_raw,
            created_at: now,
        }
    }
}

impl From<Model> for IngredientDto {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            can_be_eaten_raw: value.can_be_eaten_raw,
            created_at: value.created_at,
        }
    }
}
