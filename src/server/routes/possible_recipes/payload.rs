use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::pantry_items::dto::ListParamsDto as PantryItemsListParamsDto;
use crate::database::recipes::dto::ListParamsDto as RecipesListParamsDto;

// TODO: delete this
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

impl From<ListQueryParams> for RecipesListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        RecipesListParamsDto {
            user_id: Some(val.user_id),
            name_contains: None,
            cooking_time_mins: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PossibleRecipesResponse {
    pub recipe_ids: Vec<Uuid>,
}
