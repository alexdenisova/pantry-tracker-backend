use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use uuid::Uuid;

use db_entities::recipes::Model;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub user_id: Uuid,
    pub name: String,
    pub prep_time_mins: Option<i32>,
    pub total_time_mins: Option<i32>,
    pub link: Option<String>,
    pub instructions: Option<String>,
    pub image: Option<String>,
    pub last_cooked: Option<NaiveDate>,
    pub rating: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListParamsDto {
    pub name_contains: Option<String>,
    pub total_time_mins: Option<i32>,
    pub user_id: Option<Uuid>,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListRecipeJoinParamsDto {
    pub user_id: Uuid,
    pub ingredient_ids: Vec<Uuid>,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateDto {
    pub user_id: Uuid,
    pub name: String,
    pub prep_time_mins: Option<i32>,
    pub total_time_mins: Option<i32>,
    pub link: Option<String>,
    pub instructions: Option<String>,
    pub image: Option<String>,
    pub last_cooked: Option<NaiveDate>,
    pub rating: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Serialize, Debug, Clone, Eq)]
pub struct RecipeDto {
    pub id: Uuid,
    pub user_id: Uuid,
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

impl Hash for RecipeDto {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for RecipeDto {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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
            user_id: value.user_id,
            name: value.name,
            prep_time_mins: value.prep_time_mins,
            total_time_mins: value.total_time_mins,
            link: value.link,
            instructions: value.instructions,
            image: value.image,
            last_cooked: value.last_cooked,
            rating: value.rating,
            notes: value.notes,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Model> for RecipeDto {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            name: value.name,
            prep_time_mins: value.prep_time_mins,
            total_time_mins: value.total_time_mins,
            link: value.link,
            instructions: value.instructions,
            image: value.image,
            last_cooked: value.last_cooked,
            rating: value.rating,
            notes: value.notes,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
