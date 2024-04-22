use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use db_entities::users::Model;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub name: String,
    pub password_hash: String,
    pub admin: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateDto {
    pub name: String,
    pub password_hash: String,
    pub admin: Option<bool>,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct UserDto {
    pub id: Uuid,
    pub name: String,
    pub password_hash: String,
    pub admin: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListParamsDto {
    pub name: Option<String>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct UsersListDto {
    pub items: Vec<UserDto>,
}

impl From<CreateDto> for Model {
    fn from(value: CreateDto) -> Self {
        let now = Utc::now().naive_utc();

        Self {
            id: Uuid::new_v4(),
            password_hash: value.password_hash,
            admin: value.admin.unwrap_or(false),
            name: value.name,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Model> for UserDto {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            password_hash: value.password_hash,
            admin: value.admin,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
