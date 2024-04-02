use chrono::{NaiveDate, NaiveDateTime, Utc};
use db_entities::pantry_items::Model;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub ingredient_id: Uuid,
    pub user_id: Uuid,
    pub purchase_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
    pub quantity: Option<i32>,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateDto {
    pub purchase_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
    pub quantity: Option<i32>,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListParamsDto {
    pub max_expiration_date: Option<NaiveDate>,
    pub user_id: Option<Uuid>,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct PantryItemDto {
    pub id: Uuid,
    pub ingredient_id: Uuid,
    pub purchase_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
    pub quantity: Option<i32>,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct PantryItemsListDto {
    pub items: Vec<PantryItemDto>,
}

impl From<CreateDto> for Model {
    fn from(value: CreateDto) -> Self {
        let now = Utc::now().naive_utc();

        Self {
            id: Uuid::new_v4(),
            ingredient_id: value.ingredient_id,
            purchase_date: value.purchase_date,
            expiration_date: value.expiration_date,
            quantity: value.quantity,
            weight_grams: value.weight_grams,
            volume_milli_litres: value.volume_milli_litres,
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
            purchase_date: value.purchase_date,
            expiration_date: value.expiration_date,
            quantity: value.quantity,
            weight_grams: value.weight_grams,
            volume_milli_litres: value.volume_milli_litres,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
