use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::pantry_items::dto::{
    CreateDto, ListParamsDto, PantryItemDto, PantryItemsListDto, UpdateDto,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub ingredient_id: Uuid,
    pub purchase_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
    pub quantity: Option<i32>,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
    pub essential: bool,
    pub running_low: Option<i32>,
}

impl CreatePayload {
    pub fn into_dto(self, user_id: Uuid) -> CreateDto {
        CreateDto {
            ingredient_id: self.ingredient_id,
            user_id,
            purchase_date: self.purchase_date,
            expiration_date: self.expiration_date,
            quantity: self.quantity,
            weight_grams: self.weight_grams,
            volume_milli_litres: self.volume_milli_litres,
            essential: self.essential,
            running_low: self.running_low,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdatePayload {
    pub ingredient_id: Uuid,
    pub purchase_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
    pub quantity: Option<i32>,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
    pub essential: bool,
    pub running_low: Option<i32>,
}

impl UpdatePayload {
    pub fn into_dto(self, user_id: Uuid) -> UpdateDto {
        UpdateDto {
            ingredient_id: self.ingredient_id,
            user_id,
            purchase_date: self.purchase_date,
            expiration_date: self.expiration_date,
            quantity: self.quantity,
            weight_grams: self.weight_grams,
            volume_milli_litres: self.volume_milli_litres,
            essential: self.essential,
            running_low: self.running_low,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub with_name: Option<bool>, // TODO: always return ingredient_name
    pub name_contains: Option<String>,
    pub max_expiration_date: Option<NaiveDate>,
    pub ingredient_id: Option<Uuid>,
}

impl ListQueryParams {
    pub fn into_dto(self, user_id: Uuid) -> ListParamsDto {
        ListParamsDto {
            max_expiration_date: self.max_expiration_date,
            user_id: Some(user_id),
            ingredient_id: self.ingredient_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct PantryItemResponse {
    pub id: Uuid,
    pub ingredient_id: Uuid,
    pub user_id: Uuid,
    pub purchase_date: Option<NaiveDate>,
    pub expiration_date: Option<String>,
    pub quantity: Option<i32>,
    pub weight_grams: Option<i32>,
    pub volume_milli_litres: Option<i32>,
    pub essential: bool,
    pub running_low: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<PantryItemDto> for PantryItemResponse {
    fn from(val: PantryItemDto) -> Self {
        PantryItemResponse {
            id: val.id,
            ingredient_id: val.ingredient_id,
            user_id: val.user_id,
            purchase_date: val.purchase_date,
            expiration_date: val.expiration_date.map(|date| date.to_string()),
            quantity: val.quantity,
            weight_grams: val.weight_grams,
            volume_milli_litres: val.volume_milli_litres,
            essential: val.essential,
            running_low: val.running_low,
            created_at: val.created_at,
            updated_at: val.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct PantryItemListResponse {
    pub items: Vec<PantryItemResponse>,
}

impl From<PantryItemsListDto> for PantryItemListResponse {
    fn from(val: PantryItemsListDto) -> Self {
        PantryItemListResponse {
            items: val.items.into_iter().map(Into::into).collect(),
        }
    }
}
