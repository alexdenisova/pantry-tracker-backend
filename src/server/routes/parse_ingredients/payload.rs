use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub text: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ParsedRecipeIngredient {
    pub amount: Option<f32>,
    pub unit: Option<String>,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ParseIngredientsResponse {
    pub items: Vec<ParsedRecipeIngredient>,
}
