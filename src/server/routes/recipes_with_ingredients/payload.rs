use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::recipe_users::dto::ListParamsDto as RecipeUsersListParamsDto;

#[derive(Deserialize, Debug)]
pub struct ListPayload {
    pub ingredient_ids: Vec<Uuid>,
}

#[derive(Deserialize, Debug)]
pub struct ListQueryParams {
    pub user_id: Uuid,
}

impl From<ListQueryParams> for RecipeUsersListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        RecipeUsersListParamsDto {
            user_id: Some(val.user_id),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RecipesWithIngredientsResponse {
    pub recipe_ids: Vec<Uuid>,
}
