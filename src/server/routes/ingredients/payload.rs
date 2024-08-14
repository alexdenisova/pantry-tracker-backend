use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use titlecase::titlecase;
use uuid::Uuid;

use crate::database::ingredients::dto::{
    CreateDto, IngredientDto, IngredientsListDto, ListParamsDto,
};
use crate::server::payload::{MetadataResponse, DEFAULT_PER_PAGE};

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
    pub name: Option<String>,
    pub name_contains: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl From<ListQueryParams> for ListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        ListParamsDto {
            name: val.name,
            name_contains: val.name_contains,
            limit: val.per_page.unwrap_or(DEFAULT_PER_PAGE),
            offset: val.per_page.unwrap_or(DEFAULT_PER_PAGE) * (val.page.unwrap_or(1) - 1),
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
    #[serde(rename = "_metadata")]
    pub metadata: MetadataResponse,
    pub items: Vec<IngredientResponse>,
}

impl From<IngredientsListDto> for Vec<IngredientResponse> {
    fn from(val: IngredientsListDto) -> Self {
        val.items.into_iter().map(Into::into).collect()
    }
}

impl IngredientListResponse {
    pub fn from(items: Vec<IngredientResponse>, metadata: MetadataResponse) -> Self {
        IngredientListResponse { metadata, items }
    }
}
