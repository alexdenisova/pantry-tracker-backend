use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use titlecase::titlecase;
use uuid::Uuid;

use crate::database::categories::dto::{
    CreateDto, CategoryDto, CategoryListDto, ListParamsDto,
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
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl From<ListQueryParams> for ListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        ListParamsDto {
            name: val.name,
            limit: val.per_page.unwrap_or(DEFAULT_PER_PAGE),
            offset: val.per_page.unwrap_or(DEFAULT_PER_PAGE) * (val.page.unwrap_or(1) - 1),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
}

impl From<CategoryDto> for CategoryResponse {
    fn from(val: CategoryDto) -> Self {
        CategoryResponse {
            id: val.id,
            name: val.name,
            created_at: val.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct CategoryListResponse {
    #[serde(rename = "_metadata")]
    pub metadata: MetadataResponse,
    pub items: Vec<CategoryResponse>,
}

impl From<CategoryListDto> for Vec<CategoryResponse> {
    fn from(val: CategoryListDto) -> Self {
        val.items.into_iter().map(Into::into).collect()
    }
}

impl CategoryListResponse {
    pub fn from(items: Vec<CategoryResponse>, metadata: MetadataResponse) -> Self {
        CategoryListResponse { metadata, items }
    }
}
