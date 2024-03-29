use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ParsedRecipeIngredient {
    pub amount: Option<f32>,
    pub unit: Option<String>,
    pub name: String,
}

impl Default for ParsedRecipeIngredient {
    fn default() -> Self {
        Self {
            amount: None,
            unit: None,
            name: String::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ParseIngredientsResponse {
    pub ingredients: Vec<ParsedRecipeIngredient>,
}
