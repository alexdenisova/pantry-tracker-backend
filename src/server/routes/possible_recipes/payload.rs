use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::pantry_items::dto::ListParamsDto as PantryItemsListParamsDto;
use crate::database::recipe_users::dto::ListParamsDto as RecipeUsersListParamsDto;

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub user_id: Uuid,
}

impl From<ListQueryParams> for PantryItemsListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        PantryItemsListParamsDto {
            max_expiration_date: None,
            user_id: Some(val.user_id),
        }
    }
}

impl From<ListQueryParams> for RecipeUsersListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        RecipeUsersListParamsDto {
            user_id: Some(val.user_id),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PossibleRecipesResponse {
    pub recipe_ids: Vec<Uuid>,
}
