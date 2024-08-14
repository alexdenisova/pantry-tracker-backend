mod payload;

use axum::extract::Query;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::get,
    Router,
};
use color_eyre::eyre::eyre;
use htmlentity::entity::ICodedDataTrait;
use scraper::{Html, Selector};
use serde_json::json;
use serde_json::Value;
use std::borrow::Borrow;
use thiserror::Error;
use url::Url;
use urlencoding::decode;

use self::payload::{ListQueryParams, ParsedRecipeLinkResponse};
use crate::server::routes::errors::AppError;
use crate::server::routes::parse_ingredients::parse_ingredients;
use crate::server::routes::parse_ingredients::payload::ParsedRecipeIngredient;
use crate::server::AppState;

pub struct ParsedRecipeLinkRouter {}

impl ParsedRecipeLinkRouter {
    pub fn router() -> Router<AppState> {
        Router::new().route("/", get(ParsedRecipeLinkRouter::parse_recipe_link))
    }

    #[allow(clippy::unused_async)]
    async fn parse_recipe_link(
        State(_): State<AppState>,
        Query(query_params): Query<ListQueryParams>,
    ) -> Result<(StatusCode, Json<ParsedRecipeLinkResponse>), AppError> {
        if let Ok(link) = decode(&query_params.link) {
            let json = get_recipe_json(link.borrow()).await?;
            let name = get_name(&json);
            let prep_time_mins = get_time_field(&json, "prepTime");
            let total_time_mins = get_time_field(&json, "totalTime");
            let image = get_image(&json);
            let ingredients = get_ingredients(&json);
            let instructions = get_instructions(&json);
            return Ok((
                StatusCode::OK,
                Json(ParsedRecipeLinkResponse {
                    name,
                    prep_time_mins,
                    total_time_mins,
                    instructions,
                    image,
                    ingredients,
                }),
            ));
        }
        Err(AppError::UnprocessableEntity {
            error: eyre!("Link must be urlencoded."),
        })
    }
}

#[derive(Error, Debug)]
pub enum GetRecipeJsonError {
    #[error("Could not GET {link}: {err}")]
    LinkUnavailable { link: String, err: String },
    #[error("Could not parse respone from {link}: {err}")]
    BadFormat { link: String, err: String },
}

async fn get_recipe_json(link: &str) -> Result<Value, GetRecipeJsonError> {
    let response = match reqwest::get(link).await {
        Ok(res) => res,
        Err(e) => {
            return Err(GetRecipeJsonError::LinkUnavailable {
                link: link.to_owned(),
                err: e.to_string(),
            })
        }
    };
    let Ok(xml) = response.text().await else {
        return Err(GetRecipeJsonError::BadFormat {
            link: link.to_owned(),
            err: "No body".to_owned(),
        });
    };
    let html = Html::parse_fragment(&xml);
    let Ok(selector) = Selector::parse("script") else {
        return Err(GetRecipeJsonError::BadFormat {
            link: link.to_owned(),
            err: "No script elements".to_owned(),
        });
    };
    if let Some(element) = html.select(&selector).find(|el| {
        el.attr("type") == Some("application/ld+json")
            && el.inner_html().contains("recipeIngredient")
    }) {
        let mut json: serde_json::Value =
            serde_json::from_str(&element.text().collect::<Vec<&str>>().join("")).unwrap();
        if json.is_object() {
            if let Some(graph) = json.get("@graph") {
                json = graph.clone();
            } else {
                return Err(GetRecipeJsonError::BadFormat {
                    link: link.to_owned(),
                    err: "Could not parse recipe element".to_owned(),
                });
            }
        }
        if let Some(arr) = json.as_array() {
            if let Some(j) = arr.iter().find(|&v| {
                let Some(otype) = v.get("@type") else {
                    return false;
                };
                match otype.as_array() {
                    Some(arr) => arr.contains(&json!("Recipe")),
                    None => otype.eq(&json!("Recipe")),
                }
            }) {
                return Ok(j.clone());
            }
        }
    }
    Err(GetRecipeJsonError::BadFormat {
        link: link.to_owned(),
        err: "No recipe element".to_owned(),
    })
}

fn get_name(json: &Value) -> Option<String> {
    if let Some(name) = json.get("name") {
        if let Some(name) = name.as_str() {
            return Some(name.to_owned());
        }
    }
    None
}

fn get_time_field(json: &Value, key: &str) -> Option<u32> {
    if let Some(time) = json.get(key) {
        if let Some(time) = time.as_str() {
            return iso8601_to_mins(time);
        }
    }
    None
}

fn get_instructions(json: &Value) -> Option<String> {
    if let Some(instructions) = json.get("recipeInstructions") {
        if let Some(instructions) = instructions.as_array() {
            let mut result = String::new();
            for (num, instruct) in instructions.iter().enumerate() {
                if let Some(text) = instruct.get("text") {
                    result += &format!(
                        "{}. {}\n",
                        num + 1,
                        htmlentity::entity::decode(text.as_str().unwrap().as_bytes())
                            .to_string()
                            .unwrap()
                    );
                } else {
                    result += &format!("{}. ---\n", num + 1);
                }
            }
            return Some(result);
        }
    }
    None
}

fn iso8601_to_mins(duration: &str) -> Option<u32> {
    if let Ok(iso8601::Duration::YMDHMS {
        year: _,
        month: _,
        day,
        hour,
        minute,
        second,
        millisecond: _,
    }) = iso8601::duration(duration)
    {
        return Some(day * 60 * 60 + hour * 60 + minute + second / 60);
    }
    None
}

fn get_image(json: &Value) -> Option<Url> {
    if let Some(image) = json.get("image") {
        match image {
            Value::String(s) => return Url::parse(s).ok(),
            Value::Array(a) => match a.first() {
                Some(img) => match img {
                    Value::String(s) => return Url::parse(s).ok(),
                    Value::Object(obj) => {
                        if let Some(url) = obj.get("url") {
                            if let Some(s) = url.as_str() {
                                return Url::parse(s).ok();
                            }
                        }
                    }
                    _ => return None,
                },
                None => return None,
            },
            Value::Object(obj) => {
                if let Some(url) = obj.get("url") {
                    if let Some(s) = url.as_str() {
                        return Url::parse(s).ok();
                    }
                }
            }
            _ => return None,
        }
    }
    None
}

fn get_ingredients(json: &Value) -> Vec<ParsedRecipeIngredient> {
    if let Some(ingredients) = json.get("recipeIngredient") {
        if let Some(arr) = ingredients.as_array() {
            let mut ingredients = Vec::new();
            for a in arr {
                if let Some(s) = a.as_str() {
                    ingredients.push(s);
                }
            }
            return parse_ingredients(ingredients);
        }
    }
    Vec::new()
}
