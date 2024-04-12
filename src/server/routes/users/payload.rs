use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::users::dto::{CreateDto, ListParamsDto, UpdateDto, UserDto, UsersListDto};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub name: String,
}

impl From<CreatePayload> for CreateDto {
    fn from(val: CreatePayload) -> Self {
        CreateDto { name: val.name }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdatePayload {
    pub name: String,
}

impl From<UpdatePayload> for UpdateDto {
    fn from(val: UpdatePayload) -> Self {
        UpdateDto { name: val.name }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub name: Option<String>,
}

impl From<ListQueryParams> for ListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        ListParamsDto { name: val.name }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<UserDto> for UserResponse {
    fn from(val: UserDto) -> Self {
        UserResponse {
            id: val.id,
            name: val.name,
            created_at: val.created_at,
            updated_at: val.updated_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct UsersListResponse {
    pub items: Vec<UserResponse>,
}

impl From<UsersListDto> for UsersListResponse {
    fn from(val: UsersListDto) -> Self {
        UsersListResponse {
            items: val.items.into_iter().map(Into::into).collect(),
        }
    }
}
