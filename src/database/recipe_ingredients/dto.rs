use chrono::{NaiveDateTime, Utc};
use db_entities::recipe_ingredients::Model;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
    pub amount: f32,
    pub unit: String,
    pub optional: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListParamsDto {
    pub recipe_id: Option<Uuid>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateDto {
    pub amount: Option<f32>,
    pub unit: Option<String>,
    pub optional: Option<bool>,
}

#[derive(Serialize, Debug, Clone)]
pub struct RecipeIngredientDto {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
    pub amount: f32,
    pub unit: String,
    pub optional: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Debug)]
pub struct RecipeIngredientsListDto {
    pub items: Vec<RecipeIngredientDto>,
}

impl From<CreateDto> for Model {
    fn from(value: CreateDto) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            id: Uuid::new_v4(),
            recipe_id: value.recipe_id,
            ingredient_id: value.ingredient_id,
            amount: value.amount,
            unit: value.unit,
            optional: value.optional,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Model> for RecipeIngredientDto {
    fn from(value: Model) -> Self {
        Self {
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
