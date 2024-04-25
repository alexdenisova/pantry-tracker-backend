use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use titlecase::titlecase;
use uuid::Uuid;

use crate::database::ingredients::dto::{
    CreateDto, IngredientDto, IngredientsListDto, ListParamsDto, UpdateDto,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub name: String,
    pub can_be_eaten_raw: Option<bool>,
}

impl From<CreatePayload> for CreateDto {
    fn from(val: CreatePayload) -> Self {
        CreateDto {
            name: titlecase(&val.name),
            can_be_eaten_raw: val.can_be_eaten_raw,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdatePayload {
    pub can_be_eaten_raw: Option<bool>,
}

impl From<UpdatePayload> for UpdateDto {
    fn from(val: UpdatePayload) -> Self {
        UpdateDto {
            name: None,
            can_be_eaten_raw: val.can_be_eaten_raw,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub name: Option<String>,
}

impl From<ListQueryParams> for ListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        ListParamsDto { name: val.name }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct IngredientResponse {
    pub id: Uuid,
    pub name: String,
    pub can_be_eaten_raw: Option<bool>,
    pub created_at: NaiveDateTime,
}

impl From<IngredientDto> for IngredientResponse {
    fn from(val: IngredientDto) -> Self {
        IngredientResponse {
            id: val.id,
            name: val.name,
            can_be_eaten_raw: val.can_be_eaten_raw,
            created_at: val.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct IngredientListResponse {
    pub items: Vec<IngredientResponse>,
}

impl From<IngredientsListDto> for IngredientListResponse {
    fn from(val: IngredientsListDto) -> Self {
        IngredientListResponse {
            items: val.items.into_iter().map(Into::into).collect(),
        }
    }
}
