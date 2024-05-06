use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use titlecase::titlecase;
use uuid::Uuid;

use crate::database::ingredient_names::dto::{
    CreateDto, IngredientNameDto, IngredientNamesListDto, ListParamsDto,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub name: String,
    pub ingredient_id: Uuid,
}

impl From<CreatePayload> for CreateDto {
    fn from(val: CreatePayload) -> Self {
        CreateDto {
            name: titlecase(&val.name),
            ingredient_id: val.ingredient_id,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub ingredient_id: Option<Uuid>,
}

impl From<ListQueryParams> for ListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        ListParamsDto {
            ingredient_id: val.ingredient_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct IngredientNameResponse {
    pub id: Uuid,
    pub name: String,
    pub ingredient_id: Uuid,
    pub created_at: NaiveDateTime,
}

impl From<IngredientNameDto> for IngredientNameResponse {
    fn from(val: IngredientNameDto) -> Self {
        IngredientNameResponse {
            id: val.id,
            name: val.name,
            ingredient_id: val.ingredient_id,
            created_at: val.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct IngredientNameListResponse {
    pub items: Vec<IngredientNameResponse>,
}

impl From<IngredientNamesListDto> for IngredientNameListResponse {
    fn from(val: IngredientNamesListDto) -> Self {
        IngredientNameListResponse {
            items: val.items.into_iter().map(Into::into).collect(),
        }
    }
}
