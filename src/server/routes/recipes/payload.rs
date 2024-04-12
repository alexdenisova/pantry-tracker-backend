use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use titlecase::titlecase;
use url::Url;
use uuid::Uuid;

use crate::database::recipes::dto::{
    CreateDto, ListParamsDto, RecipeDto, RecipesListDto, UpdateDto,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub name: String,
    pub cooking_time_mins: Option<i32>,
    pub link: Option<Url>,
    pub instructions: Option<String>,
    pub image: Option<Url>,
    // pub calories
}

impl From<CreatePayload> for CreateDto {
    fn from(val: CreatePayload) -> Self {
        CreateDto {
            name: titlecase(&val.name),
            cooking_time_mins: val.cooking_time_mins,
            link: val.link.map(|url| url.to_string()),
            instructions: val.instructions,
            image: val.image.map(|url| url.to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdatePayload {
    pub name: String,
    pub cooking_time_mins: Option<i32>,
    pub link: Option<Url>,
    pub instructions: Option<String>,
    pub image: Option<Url>,
}

impl From<UpdatePayload> for UpdateDto {
    fn from(val: UpdatePayload) -> Self {
        UpdateDto {
            name: val.name,
            cooking_time_mins: val.cooking_time_mins,
            link: val.link.map(|url| url.to_string()),
            instructions: val.instructions,
            image: val.image.map(|url| url.to_string()),
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub name_contains: Option<String>,
    pub cooking_time_mins: Option<i32>,
    pub ingredient_ids: Option<String>, // urlencoded array of ingredient_ids
}

impl ListQueryParams {
    pub fn into_dto(self, user_id: Option<Uuid>) -> ListParamsDto {
        ListParamsDto {
            name_contains: self.name_contains,
            cooking_time_mins: self.cooking_time_mins,
            user_id,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct RecipeResponse {
    pub id: Uuid,
    pub name: String,
    pub cooking_time_mins: Option<i32>,
    pub link: Option<String>,
    pub instructions: Option<String>,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<RecipeDto> for RecipeResponse {
    fn from(val: RecipeDto) -> Self {
        RecipeResponse {
            id: val.id,
            name: val.name,
            cooking_time_mins: val.cooking_time_mins,
            link: val.link,
            instructions: val.instructions,
            image: val.image,
            created_at: val.created_at,
            updated_at: val.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct RecipeListResponse {
    pub items: Vec<RecipeResponse>,
}

impl From<RecipesListDto> for RecipeListResponse {
    fn from(val: RecipesListDto) -> Self {
        RecipeListResponse {
            items: val.items.into_iter().map(Into::into).collect(),
        }
    }
}
