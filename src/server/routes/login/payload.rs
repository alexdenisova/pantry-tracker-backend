use serde::{Deserialize, Serialize};

use crate::database::users::dto::ListParamsDto;

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: Option<String>,
}

impl From<LoginPayload> for ListParamsDto {
    fn from(val: LoginPayload) -> Self {
        ListParamsDto {
            name: Some(val.username),
        }
    }
}
