use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::recipe_categories::dto::{
    CreateDto, ListParamsDto, RecipeCategoryDto, RecipeCategoryJoinDto, RecipeCategoryListDto,
};
use crate::server::payload::{MetadataResponse, DEFAULT_PER_PAGE};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub recipe_id: Uuid,
    pub category_id: Uuid,
}

impl From<CreatePayload> for CreateDto {
    fn from(val: CreatePayload) -> Self {
        CreateDto {
            recipe_id: val.recipe_id,
            category_id: val.category_id,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub recipe_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub name_contains: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl ListQueryParams {
    pub fn into_dto(self, user_id: Uuid) -> ListParamsDto {
        ListParamsDto {
            recipe_id: self.recipe_id,
            category_id: self.category_id,
            user_id: Some(user_id),
            name_contains: self.name_contains,
            limit: self.per_page.unwrap_or(DEFAULT_PER_PAGE),
            offset: self.per_page.unwrap_or(DEFAULT_PER_PAGE) * (self.page.unwrap_or(1) - 1),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RecipeCategoryResponse {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub category_id: Uuid,
    pub created_at: NaiveDateTime,
}

impl From<RecipeCategoryDto> for RecipeCategoryResponse {
    fn from(val: RecipeCategoryDto) -> Self {
        RecipeCategoryResponse {
            id: val.id,
            recipe_id: val.recipe_id,
            category_id: val.category_id,
            created_at: val.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RecipeCategoryJoinResponse {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub created_at: NaiveDateTime,
}

impl From<RecipeCategoryJoinDto> for RecipeCategoryResponse {
    fn from(val: RecipeCategoryJoinDto) -> Self {
        RecipeCategoryResponse {
            id: val.id,
            recipe_id: val.recipe_id,
            category_id: val.category_id,
            created_at: val.created_at,
        }
    }
}

impl From<RecipeCategoryJoinDto> for RecipeCategoryJoinResponse {
    fn from(val: RecipeCategoryJoinDto) -> Self {
        RecipeCategoryJoinResponse {
            id: val.id,
            recipe_id: val.recipe_id,
            category_id: val.category_id,
            category_name: val.category_name,
            created_at: val.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RecipeCategoryListResponse {
    #[serde(rename = "_metadata")]
    pub metadata: MetadataResponse,
    pub items: Vec<RecipeCategoryJoinResponse>,
}

impl From<RecipeCategoryListDto> for Vec<RecipeCategoryJoinResponse> {
    fn from(val: RecipeCategoryListDto) -> Self {
        val.items.into_iter().map(Into::into).collect()
    }
}

impl RecipeCategoryListResponse {
    pub fn from(items: Vec<RecipeCategoryJoinResponse>, metadata: MetadataResponse) -> Self {
        RecipeCategoryListResponse { metadata, items }
    }
}
