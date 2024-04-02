use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::recipes::dto::{
    CreateDto, ListParamsDto, RecipeDto, RecipesListDto, UpdateDto,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub name: String,
    pub cooking_time_mins: Option<i32>,
    pub link: Option<String>,
    pub instructions: Option<String>,
    pub image: Option<String>,
    // pub calories
}

impl From<CreatePayload> for CreateDto {
    fn from(val: CreatePayload) -> Self {
        CreateDto {
            name: val.name,
            cooking_time_mins: val.cooking_time_mins,
            link: val.link,
            instructions: val.instructions,
            image: val.image,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdatePayload {
    pub name: Option<String>,
    pub cooking_time_mins: Option<i32>,
    pub link: Option<String>,
    pub instructions: Option<String>,
    pub image: Option<String>,
}

impl From<UpdatePayload> for UpdateDto {
    fn from(val: UpdatePayload) -> Self {
        UpdateDto {
            name: val.name,
            cooking_time_mins: val.cooking_time_mins,
            link: val.link,
            instructions: val.instructions,
            image: val.image,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub name_contains: Option<String>,
    pub cooking_time_mins: Option<i32>,
}

impl From<ListQueryParams> for ListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        ListParamsDto {
            name_contains: val.name_contains,
            cooking_time_mins: val.cooking_time_mins,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
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
