use crate::server::{routes::parse_ingredients::payload::ParsedRecipeIngredient, AppState};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::get,
    Router,
};
use thiserror::Error;

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

use self::payload::ParseIngredientsResponse;

mod payload;

pub struct ParseIngredientsRouter {}

impl ParseIngredientsRouter {
    pub fn get() -> Router<AppState> {
        Router::new().route("/", get(ParseIngredientsRouter::parse_ingredients))
    }

    #[allow(clippy::unused_async)]
    async fn parse_ingredients(
        State(_): State<AppState>,
        payload: String,
    ) -> (StatusCode, Json<ParseIngredientsResponse>) {
        let ingredients = payload.replace(|c: char| !c.is_ascii() && !c.is_alphanumeric(), "");
        let mut parsed = Vec::new();
        for line in ingredients.split('\n') {
            let line = line.trim();
            let mut split = line.splitn(3, ' ');
            let mut ingredient = ParsedRecipeIngredient::default();
            if let Some(word_1) = split.next() {
                ingredient.amount = parse_amount(word_1.trim()).ok();
            }
            if let Some(word_2) = split.next() {
                ingredient.unit = parse_unit(word_2.trim()).ok();
            }
            if let Some(word_3) = split.next() {
                ingredient.name = parse_ingredient(word_3.trim());
            }
            if ingredient.amount.is_none()
                || ingredient.unit.is_none()
                || ingredient.name.is_empty()
            {
                log::debug!("Failed to parse ingredient: {}", line);
                ingredient.name = line.to_owned();
                parsed.push(ingredient);
                continue;
            }
            log::debug!("Parsed ingredient: {:?}", ingredient);
            parsed.push(ingredient);
        }
        (
            StatusCode::OK,
            Json(ParseIngredientsResponse {
                ingredients: parsed,
            }),
        )
    }
}

#[derive(Error, Debug)]
pub enum ParseAmountError {
    #[error("Could not parse amount: unknown character {c}")]
    UnknownCharacter { c: char },
    #[error("Could not find amount")]
    NoAmount,
}

fn parse_amount(amount: &str) -> Result<f32, ParseAmountError> {
    if amount.starts_with(|c: char| c.is_numeric()) {
        let mut calc_amount = String::new();
        for c in amount.chars() {
            match c {
                '0'..='9' => calc_amount += c.to_string().as_str(),
                '½' => calc_amount += ".5",
                '⅓' => calc_amount += ".33",
                '¼' => calc_amount += ".25",
                _ => return Err(ParseAmountError::UnknownCharacter { c }),
            };
        }
        Ok(calc_amount.parse().unwrap())
    } else {
        Err(ParseAmountError::NoAmount)
    }
}

#[derive(Error, Debug)]
pub enum ParseUnitError {
    #[error("Could not parse unit: unknown unit {unit}")]
    UnknownUnit { unit: String },
}

fn parse_unit(unit: &str) -> Result<String, ParseUnitError> {
    if MEASUREMENTS.contains(&unit) {
        return Ok(unit.to_owned());
    }
    Err(ParseUnitError::UnknownUnit {
        unit: unit.to_owned(),
    })
}

fn parse_ingredient(string: &str) -> String {
    let mut ingredient = String::new();
    for c in string.chars() {
        if c.is_alphabetic() || c == ' ' {
            ingredient += c.to_string().as_str();
        } else {
            break;
        }
    }
    ingredient
}
