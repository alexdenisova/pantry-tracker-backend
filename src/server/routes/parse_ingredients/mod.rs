pub mod payload;

use axum::extract::Query;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::get,
    Router,
};
use color_eyre::eyre::eyre;
use regex::Regex;
use thiserror::Error;
use urlencoding::decode;

use self::payload::{ListQueryParams, ParseIngredientsResponse, ParsedRecipeIngredient};
use crate::server::routes::errors::AppError;
use crate::server::AppState;

const MEASUREMENTS: [&str; 16] = [
    "cup",
    "tablespoon",
    "tbsp",
    "teaspoon",
    "tsp",
    "ounces",
    "oz",
    "lb",
    "pound",
    "gram",
    "g",
    "kilogram",
    "kg",
    "milliliter",
    "millilitre",
    "ml",
];
const INGREDIENT_REGEX: &str = r"^((\d*)( |\.|\/)?(\d*\/\d*|\d+|½|⅔|⅓|¼)?) ?([a-z]*) ?(.*)$";

pub struct ParseIngredientsRouter {}

impl ParseIngredientsRouter {
    pub fn router() -> Router<AppState> {
        Router::new().route("/", get(ParseIngredientsRouter::parse_ingredients))
    }

    #[allow(clippy::unused_async)]
    async fn parse_ingredients(
        State(_): State<AppState>,
        Query(query_params): Query<ListQueryParams>,
    ) -> Result<(StatusCode, Json<ParseIngredientsResponse>), AppError> {
        if let Ok(input) = decode(&query_params.text) {
            let ingredients = input.replace(|c: char| !c.is_ascii() && !c.is_alphanumeric(), "");
            let parsed = parse_ingredients(ingredients.split('\n').collect());
            return Ok((
                StatusCode::OK,
                Json(ParseIngredientsResponse { items: parsed }),
            ));
        }
        Err(AppError::UnprocessableEntity {
            error: eyre!("Ingredients must be urlencoded."),
        })
    }
}

pub fn parse_ingredients(ingredients: Vec<&str>) -> Vec<ParsedRecipeIngredient> {
    let mut parsed = Vec::new();
    for ingredient in ingredients {
        if !ingredient.is_empty() {
            if let Some(ingredient) = parse_ingredient(ingredient) {
                parsed.push(ingredient);
            } else {
                log::debug!("Failed to parse ingredient: {}", ingredient);
                parsed.push(ParsedRecipeIngredient {
                    amount: None,
                    unit: None,
                    name: ingredient.to_owned(),
                });
            }
        }
    }
    parsed
}

fn parse_ingredient(line: &str) -> Option<ParsedRecipeIngredient> {
    let line = line.trim();
    let mut ingredient = ParsedRecipeIngredient::default();
    let re = Regex::new(INGREDIENT_REGEX).unwrap();
    if let Some(caps) = re.captures(line) {
        ingredient.amount = parse_amount(line, &re);

        if let Some(word_2) = caps.get(5) {
            if let Ok(unit) = parse_unit(word_2.as_str()) {
                ingredient.unit = Some(unit);
                if let Some(word_3) = caps.get(6) {
                    ingredient.name = word_3.as_str().to_owned();
                } else {
                    return None;
                }
            } else {
                ingredient.unit = None;
                if let Some(word_3) = caps.get(6) {
                    ingredient.name = [word_2.as_str(), word_3.as_str()].join(" ");
                } else {
                    ingredient.name = word_2.as_str().to_owned();
                }
            }
        } else {
            return None;
        }
        log::debug!("Parsed ingredient: {:?}", ingredient);
        return Some(ingredient);
    }
    None
}

fn parse_amount(line: &str, re: &Regex) -> Option<String> {
    let caps = re.captures(line).unwrap();
    if let Some(amount) = caps.get(1) {
        let amount = amount.as_str().trim().to_string();
        if !amount.is_empty() {
            return Some(amount);
        }
    }
    None
}

#[derive(Error, Debug)]
pub enum ParseUnitError {
    #[error("Could not parse unit: unknown unit {unit}")]
    Unknown { unit: String },
}

fn parse_unit(unit: &str) -> Result<String, ParseUnitError> {
    if MEASUREMENTS.iter().any(|m| {
        unit.eq(*m) || unit.eq(&((*m).to_owned() + "s")) || unit.eq(&((*m).to_owned() + "."))
    }) {
        return Ok(unit.to_owned());
    }

    Err(ParseUnitError::Unknown {
        unit: unit.to_owned(),
    })
}
