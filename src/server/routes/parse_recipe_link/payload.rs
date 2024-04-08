use serde::{Deserialize, Serialize};
use url::Url;
use crate::server::routes::parse_ingredients::payload::ParsedRecipeIngredient;

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub link: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ParsedRecipeLinkResponse {
    pub name: Option<String>,
    pub cooking_time_mins: Option<u32>,
    pub instructions: Option<String>,
    pub image: Option<Url>,
    pub ingredients: Vec<ParsedRecipeIngredient>,
}
