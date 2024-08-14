use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use titlecase::titlecase;
use url::Url;
use uuid::Uuid;

use crate::database::recipes::dto::{
    CreateDto, ListParamsDto, ListRecipeJoinParamsDto, RecipeDto, RecipesListDto, UpdateDto
};
use crate::server::payload::{MetadataResponse, DEFAULT_PER_PAGE};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub name: String,
    pub prep_time_mins: Option<i32>,
    pub total_time_mins: Option<i32>,
    pub link: Option<Url>,
    pub instructions: Option<String>,
    pub image: Option<Url>,
    pub last_cooked: Option<NaiveDate>,
    pub rating: Option<u8>,
    pub notes: Option<String>,
    // pub calories
}

impl CreatePayload {
    pub fn into_dto(self, user_id: Uuid) -> CreateDto {
        CreateDto {
            user_id,
            name: titlecase(&self.name),
            prep_time_mins: self.prep_time_mins,
            total_time_mins: self.total_time_mins,
            link: self.link.map(|url| url.to_string()),
            instructions: self.instructions,
            image: self.image.map(|url| url.to_string()),
            last_cooked: self.last_cooked,
            rating: self.rating.map(std::convert::Into::into),
            notes: self.notes,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdatePayload {
    pub name: String,
    pub prep_time_mins: Option<i32>,
    pub total_time_mins: Option<i32>,
    pub link: Option<String>,
    pub instructions: Option<String>,
    pub image: Option<String>,
    pub last_cooked: Option<NaiveDate>,
    pub rating: Option<u8>,
    pub notes: Option<String>,
}

impl UpdatePayload {
    pub fn into_dto(self, user_id: Uuid) -> UpdateDto {
        UpdateDto {
            user_id,
            name: self.name,
            prep_time_mins: self.prep_time_mins,
            total_time_mins: self.total_time_mins,
            link: self.link.map(|url| url.to_string()),
            instructions: self.instructions,
            image: self.image.map(|url| url.to_string()),
            last_cooked: self.last_cooked,
            rating: self.rating.map(std::convert::Into::into),
            notes: self.notes,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub name_contains: Option<String>,
    pub total_time_mins: Option<i32>,
    pub ingredient_ids: Option<String>, // urlencoded array of ingredient_ids
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl ListQueryParams {
    pub fn into_dto(self, user_id: Uuid) -> ListParamsDto {
        ListParamsDto {
            name_contains: self.name_contains,
            total_time_mins: self.total_time_mins,
            user_id: Some(user_id),
            limit: self.per_page.unwrap_or(DEFAULT_PER_PAGE),
            offset: self.per_page.unwrap_or(DEFAULT_PER_PAGE) * (self.page.unwrap_or(1) - 1),
        }
    }
    pub fn into_join_dto(self, user_id: Uuid, ingredient_ids: Vec<Uuid>) -> ListRecipeJoinParamsDto {
        ListRecipeJoinParamsDto {
            user_id,
            ingredient_ids,
            limit: self.per_page.unwrap_or(DEFAULT_PER_PAGE),
            offset: self.per_page.unwrap_or(DEFAULT_PER_PAGE) * (self.page.unwrap_or(1) - 1),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct RecipeResponse {
    pub id: Uuid,
    pub name: String,
    pub prep_time_mins: Option<i32>,
    pub total_time_mins: Option<i32>,
    pub link: Option<String>,
    pub instructions: Option<String>,
    pub image: Option<String>,
    pub last_cooked: Option<NaiveDate>,
    pub rating: Option<i32>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<RecipeDto> for RecipeResponse {
    fn from(val: RecipeDto) -> Self {
        RecipeResponse {
            id: val.id,
            name: val.name,
            prep_time_mins: val.prep_time_mins,
            total_time_mins: val.total_time_mins,
            link: val.link,
            instructions: val.instructions,
            image: val.image,
            last_cooked: val.last_cooked,
            rating: val.rating,
            notes: val.notes,
            created_at: val.created_at,
            updated_at: val.updated_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct RecipeListResponse {
    #[serde(rename = "_metadata")]
    pub metadata: MetadataResponse,
    pub items: Vec<RecipeResponse>,
}

impl From<RecipesListDto> for Vec<RecipeResponse> {
    fn from(val: RecipesListDto) -> Self {
        val.items.into_iter().map(Into::into).collect()
    }
}

impl RecipeListResponse {
    pub fn from(items: Vec<RecipeResponse>, metadata: MetadataResponse) -> Self {
        RecipeListResponse { metadata, items }
    }
}
