pub mod payload;

use self::payload::{ListQueryParams, ParseIngredientsResponse, ParsedRecipeIngredient};
use crate::server::AppState;

use axum::extract::Query;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::get,
    Router,
};
use regex::Regex;
use thiserror::Error;
use urlencoding::decode;

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
    pub fn get() -> Router<AppState> {
        Router::new().route("/", get(ParseIngredientsRouter::parse_ingredients))
    }

    #[allow(clippy::unused_async)]
    async fn parse_ingredients(
        State(_): State<AppState>,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<ParseIngredientsResponse>>) {
        if let Ok(input) = decode(&query_params.text) {
            let ingredients = input.replace(|c: char| !c.is_ascii() && !c.is_alphanumeric(), "");
            let parsed = parse_ingredients(ingredients.split('\n').collect());
            return (
                StatusCode::OK,
                Json(Some(ParseIngredientsResponse { items: parsed })),
            );
        }
        (StatusCode::UNPROCESSABLE_ENTITY, Json(None))
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

// #[derive(Error, Debug)]
// pub enum ParseAmountError {
//     #[error("Could not parse amount: {err}")]
//     Unknown { err: String },
// }

// fn parse_amount(line: &str, re: &Regex) -> Result<f32, ParseAmountError> {
//     let caps = re.captures(line).unwrap();
//     let mut calc_amount = String::new();
//     if let Some(whole_part) = caps.get(1) {
//         let whole_part = whole_part.as_str();
//         if let Some(cap_2) = caps.get(2) {
//             let cap_2 = cap_2.as_str();
//             if cap_2 == "/" {
//                 // if a fraction
//                 if let Some(denominator) = caps.get(3) {
//                     let numerator: f32 = whole_part.parse().map_err(|err: ParseFloatError| {
//                         ParseAmountError::Unknown {
//                             err: err.to_string(),
//                         }
//                     })?;
//                     let denominator: f32 =
//                         denominator
//                             .as_str()
//                             .parse()
//                             .map_err(|err: ParseFloatError| ParseAmountError::Unknown {
//                                 err: err.to_string(),
//                             })?;
//                     return Ok(numerator / denominator);
//                 }
//                 return Err(ParseAmountError::Unknown {
//                     err: "strange format".to_owned(),
//                 });
//             }
//         }
//         calc_amount += whole_part;
//         calc_amount += ".";
//         if let Some(decimal_part) = caps.get(3) {
//             calc_amount += &decimal_to_string(decimal_part.as_str());
//         } else {
//             calc_amount += "0";
//         }
//     }
//     calc_amount
//         .parse()
//         .map_err(|err: ParseFloatError| ParseAmountError::Unknown {
//             err: err.to_string(),
//         })
// }

// fn decimal_to_string(decimal_part: &str) -> String {
//     match decimal_part {
//         "½" => "5".to_string(),
//         "⅔" => "67".to_string(),
//         "⅓" => "33".to_string(),
//         "¼" => "25".to_string(),
//         d => {
//             let re = Regex::new(r"^(\d+)(\/)?(\d+)?").unwrap();
//             if re.is_match(d) {
//                 parse_amount(d, &re)
//                     .unwrap()
//                     .to_string()
//                     .trim_start_matches("0.")
//                     .to_string()
//             } else {
//                 d.to_string()
//             }
//         }
//     }
// }
