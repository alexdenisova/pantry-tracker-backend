use chrono::{NaiveDate, NaiveDateTime, Utc};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use db_entities::pantry_items::Model;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub ingredient_id: Uuid,
    pub user_id: Uuid,
    pub expiration_date: Option<NaiveDate>,
    pub quantity: Option<i32>,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
    pub essential: bool,
    pub running_low: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateDto {
    pub ingredient_id: Uuid,
    pub user_id: Uuid,
    pub expiration_date: Option<NaiveDate>,
    pub quantity: Option<i32>,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
    pub essential: bool,
    pub running_low: Option<i32>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListParamsDto {
    pub user_id: Option<Uuid>,
    pub ingredient_id: Option<Uuid>,
    pub name_contains: Option<String>,
    pub max_expiration_date: Option<NaiveDate>,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct PantryItemDto {
    pub id: Uuid,
    pub ingredient_id: Uuid,
    pub expiration_date: Option<NaiveDate>,
    pub quantity: Option<i32>,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
    pub essential: bool,
    pub running_low: Option<i32>,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<CreateDto> for Model {
    fn from(value: CreateDto) -> Self {
        let now = Utc::now().naive_utc();

        Self {
            id: Uuid::new_v4(),
            ingredient_id: value.ingredient_id,
            expiration_date: value.expiration_date,
            quantity: value.quantity,
            weight_grams: value.weight_grams,
            volume_milli_litres: value.volume_milli_litres,
            essential: value.essential,
            running_low: value.running_low,
            user_id: value.user_id,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Model> for PantryItemDto {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            ingredient_id: value.ingredient_id,
            expiration_date: value.expiration_date,
            quantity: value.quantity,
            weight_grams: value.weight_grams,
            volume_milli_litres: value.volume_milli_litres,
            essential: value.essential,
            running_low: value.running_low,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq, FromQueryResult)]
pub struct PantryItemJoinDto {
    pub id: Uuid,
    pub ingredient_id: Uuid,
    pub ingredient_name: String,
    pub purchase_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
    pub quantity: Option<i32>,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
    pub essential: bool,
    pub running_low: Option<i32>,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct PantryItemsListDto {
    pub items: Vec<PantryItemJoinDto>,
}
