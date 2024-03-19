use chrono::{NaiveDateTime, Utc};
use entities::users::Model;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateDto {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateDto {
    pub name: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListParamsDto {
    pub predicate: Option<String>,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct UserDto {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
          created_at: value.created_at,
          updated_at: value.updated_at,
      }
  }
}
