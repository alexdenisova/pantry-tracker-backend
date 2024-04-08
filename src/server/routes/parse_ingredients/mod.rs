use std::num::ParseFloatError;

use crate::server::routes::parse_ingredients::payload::ListQueryParams;
use crate::server::{routes::parse_ingredients::payload::ParsedRecipeIngredient, AppState};
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
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<ParseIngredientsResponse>>) {
        if let Ok(input) = decode(&query_params.text) {
            let ingredients = input.replace(|c: char| !c.is_ascii() && !c.is_alphanumeric(), "");
            let mut parsed = Vec::new();
            for line in ingredients.split('\n') {
                let line = line.trim();
                let re = Regex::new(r"^(\d*)(\.|\/)?(\d+|½|⅔|⅓|¼)? (\S*) ?(.+)$").unwrap();
                let caps = re.captures(line).unwrap();
                let mut ingredient = ParsedRecipeIngredient::default();
                if let Ok(amount) = parse_amount(line, &re) {
                    ingredient.amount = Some(amount);
                } else {
                    failed_to_parse(line, ingredient, &mut parsed);
                    continue;
                }

                if let Some(word_2) = caps.get(4) {
                    if let Ok(unit) = parse_unit(word_2.as_str()) {
                        ingredient.unit = Some(unit);
                        if let Some(word_3) = caps.get(5) {
                            ingredient.name = parse_ingredient(word_3.as_str());
                        } else {
                            failed_to_parse(line, ingredient, &mut parsed);
                            continue;
                        }
                    } else {
                        ingredient.unit = None;
                        if let Some(word_3) = caps.get(5) {
                            ingredient.name =
                                parse_ingredient(&[word_2.as_str(), word_3.as_str()].join(" "));
                        } else {
                            ingredient.name = parse_ingredient(word_2.as_str());
                        }
                    }
                } else {
                    failed_to_parse(line, ingredient, &mut parsed);
                    continue;
                }

                log::debug!("Parsed ingredient: {:?}", ingredient);
                parsed.push(ingredient);
            }
            (
                StatusCode::OK,
                Json(Some(ParseIngredientsResponse { items: parsed })),
            )
        } else {
            (StatusCode::UNPROCESSABLE_ENTITY, Json(None))
        }
    }
}

fn failed_to_parse(
    line: &str,
    mut ingredient: ParsedRecipeIngredient,
    parsed: &mut Vec<ParsedRecipeIngredient>,
) {
    log::debug!("Failed to parse ingredient: {}", line);
    ingredient.name = line.to_owned();
    (*parsed).push(ingredient);
}

#[derive(Error, Debug)]
pub enum ParseAmountError {
    #[error("Could not find amount")]
    NoAmount,
    #[error("Could not parse amount: {err}")]
    Other { err: String },
}

fn parse_amount(line: &str, re: &Regex) -> Result<f32, ParseAmountError> {
    let caps = re.captures(line).unwrap();
    let mut calc_amount = String::new();
    if let Some(whole_part) = caps.get(1) {
        let whole_part = whole_part.as_str();
        if let Some(fraction) = caps.get(2) {
            let fraction = fraction.as_str();
            if fraction == "/" {
                // if a fraction
                if let Some(denominator) = caps.get(3) {
                    let numerator: f32 = whole_part.parse().map_err(|err: ParseFloatError| {
                        ParseAmountError::Other {
                            err: err.to_string(),
                        }
                    })?;
                    let denominator: f32 =
                        denominator
                            .as_str()
                            .parse()
                            .map_err(|err: ParseFloatError| ParseAmountError::Other {
                                err: err.to_string(),
                            })?;
                    return Ok(numerator / denominator);
                }
                return Err(ParseAmountError::Other {
                    err: "strange format".to_owned(),
                });
            }
        } else {
            calc_amount += whole_part;
            calc_amount += ".";
            if let Some(decimal_part) = caps.get(3) {
                calc_amount += &decimal_to_string(decimal_part.as_str());
            } else {
                calc_amount += "0";
            }
        }
    } else {
        calc_amount += "0";
        calc_amount += ".";
        if let Some(decimal_part) = caps.get(3) {
            calc_amount += &decimal_to_string(decimal_part.as_str());
        } else {
            return Err(ParseAmountError::NoAmount);
        }
    }
    calc_amount
        .parse()
        .map_err(|err: ParseFloatError| ParseAmountError::Other {
            err: err.to_string(),
        })
}

fn decimal_to_string(decimal_part: &str) -> String {
    match decimal_part {
        "½" => "5",
        "⅔" => "67",
        "⅓" => "33",
        "¼" => "25",
        d => d,
    }
    .to_string()
}

#[derive(Error, Debug)]
pub enum ParseUnitError {
    #[error("Could not parse unit: unknown unit {unit}")]
    UnknownUnit { unit: String },
}

fn parse_unit(unit: &str) -> Result<String, ParseUnitError> {
    if MEASUREMENTS.iter().any(|m| {
        unit.eq(*m) || unit.eq(&((*m).to_owned() + "s")) || unit.eq(&((*m).to_owned() + "."))
    }) {
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
