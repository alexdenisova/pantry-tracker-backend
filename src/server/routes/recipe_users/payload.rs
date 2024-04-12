use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::recipe_users::dto::{
    CreateDto, ListParamsDto, RecipeUserDto, RecipeUsersListDto,
};

// TODO:
#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub recipe_id: Uuid,
    pub user_id: Uuid,
}

impl From<CreatePayload> for CreateDto {
    fn from(val: CreatePayload) -> Self {
        CreateDto {
            recipe_id: val.recipe_id,
            user_id: val.user_id,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub user_id: Option<Uuid>,
}

impl From<ListQueryParams> for ListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        ListParamsDto {
            user_id: val.user_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct RecipeUserResponse {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
}

impl From<RecipeUserDto> for RecipeUserResponse {
    fn from(val: RecipeUserDto) -> Self {
        RecipeUserResponse {
            id: val.id,
            recipe_id: val.recipe_id,
            user_id: val.user_id,
            created_at: val.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct RecipeUserListResponse {
    pub items: Vec<RecipeUserResponse>,
}

impl From<RecipeUsersListDto> for RecipeUserListResponse {
    fn from(val: RecipeUsersListDto) -> Self {
        RecipeUserListResponse {
            items: val.items.into_iter().map(Into::into).collect(),
        }
    }
}
