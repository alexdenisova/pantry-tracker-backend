use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use db_entities::ingredient_names::Model;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub name: String,
    pub ingredient_id: Uuid,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListParamsDto {
    pub ingredient_id: Option<Uuid>,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct IngredientNameDto {
    pub id: Uuid,
    pub name: String,
    pub ingredient_id: Uuid,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct IngredientNamesListDto {
    pub items: Vec<IngredientNameDto>,
}

impl From<CreateDto> for Model {
    fn from(value: CreateDto) -> Self {
        let now = Utc::now().naive_utc();

        Self {
            id: Uuid::new_v4(),
            name: value.name,
            ingredient_id: value.ingredient_id,
            created_at: now,
        }
    }
}

impl From<Model> for IngredientNameDto {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            ingredient_id: value.ingredient_id,
            created_at: value.created_at,
        }
    }
}
