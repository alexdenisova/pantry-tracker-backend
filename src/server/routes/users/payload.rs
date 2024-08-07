use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::users::dto::{CreateDto, ListParamsDto, UpdateDto, UserDto, UsersListDto};
use crate::server::payload::{MetadataResponse, DEFAULT_PER_PAGE};
use crate::server::routes::utils::hash_password;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub name: String,
    pub password: String,
    pub admin: Option<bool>,
}

impl From<CreatePayload> for CreateDto {
    fn from(val: CreatePayload) -> Self {
        CreateDto {
            name: val.name,
            password_hash: hash_password(&val.password),
            admin: val.admin,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdatePayload {
    pub name: String,
    pub password: String,
    pub admin: Option<bool>,
}

impl From<UpdatePayload> for UpdateDto {
    fn from(val: UpdatePayload) -> Self {
        UpdateDto {
            name: val.name,
            password_hash: hash_password(&val.password),
            admin: val.admin,
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
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub password_hash: String,
    pub admin: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<UserDto> for UserResponse {
    fn from(val: UserDto) -> Self {
        UserResponse {
            id: val.id,
            name: val.name,
            password_hash: val.password_hash,
            admin: val.admin,
            created_at: val.created_at,
            updated_at: val.updated_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct UsersListResponse {
    #[serde(rename = "_metadata")]
    pub metadata: MetadataResponse,
    pub items: Vec<UserResponse>,
}

impl From<UsersListDto> for Vec<UserResponse> {
    fn from(val: UsersListDto) -> Self {
        val.items.into_iter().map(Into::into).collect()
    }
}

impl UsersListResponse {
    pub fn from(items: Vec<UserResponse>, metadata: MetadataResponse) -> Self {
        UsersListResponse { metadata, items }
    }
}
