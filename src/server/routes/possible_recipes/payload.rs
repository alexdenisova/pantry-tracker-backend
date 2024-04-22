use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct PossibleRecipesResponse {
    pub recipe_ids: Vec<Uuid>,
}
