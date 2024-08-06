use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use titlecase::titlecase;
use uuid::Uuid;

use crate::database::ingredients::dto::{
    CreateDto, IngredientDto, IngredientsListDto, ListParamsDto,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub name: String,
}

impl From<CreatePayload> for CreateDto {
    fn from(val: CreatePayload) -> Self {
        CreateDto {
            name: titlecase(&val.name),
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub name: Option<String>, // TODO: add name_contains for filter
    pub name_contains: Option<String>,
}

impl From<ListQueryParams> for ListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        ListParamsDto {
            name: val.name,
            name_contains: val.name_contains,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct IngredientResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
}

impl From<IngredientDto> for IngredientResponse {
    fn from(val: IngredientDto) -> Self {
        IngredientResponse {
            id: val.id,
            name: val.name,
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
