use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use db_entities::recipes::Model;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub name: String,
    pub cooking_time_mins: Option<i32>,
    pub link: Option<String>,
    pub instructions: Option<String>,
    pub image: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListParamsDto {
    pub name_contains: Option<String>,
    pub cooking_time_mins: Option<i32>,
    pub user_id: Option<Uuid>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateDto {
    pub name: String,
    pub cooking_time_mins: Option<i32>,
    pub link: Option<String>,
    pub instructions: Option<String>,
    pub image: Option<String>,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct RecipeDto {
    pub id: Uuid,
    pub name: String,
    pub cooking_time_mins: Option<i32>,
    pub link: Option<String>,
    pub instructions: Option<String>,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct RecipesListDto {
    pub items: Vec<RecipeDto>,
}

impl From<CreateDto> for Model {
    fn from(value: CreateDto) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            id: Uuid::new_v4(),
            name: value.name,
            cooking_time_mins: value.cooking_time_mins,
            link: value.link,
            instructions: value.instructions,
            image: value.image,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Model> for RecipeDto {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            cooking_time_mins: value.cooking_time_mins,
            link: value.link,
            instructions: value.instructions,
            image: value.image,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
