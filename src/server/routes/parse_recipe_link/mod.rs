mod payload;

use self::payload::{ListQueryParams, ParsedRecipeLinkResponse};
use crate::server::routes::parse_ingredients::parse_ingredients;
use crate::server::routes::parse_ingredients::payload::ParsedRecipeIngredient;
use crate::server::AppState;

use axum::extract::Query;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::get,
    Router,
};
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::json;
use serde_json::Value;
use std::num::ParseFloatError;
use thiserror::Error;
use url::Url;
use urlencoding::decode;

pub struct ParsedRecipeLinkRouter {}

impl ParsedRecipeLinkRouter {
    pub fn get() -> Router<AppState> {
        Router::new().route("/", get(ParsedRecipeLinkRouter::parse_recipe_link))
    }

    #[allow(clippy::unused_async)]
    async fn parse_recipe_link(
        State(_): State<AppState>,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<ParsedRecipeLinkResponse>>) {
        if let Ok(link) = decode(&query_params.link) {
            let xml = reqwest::blocking::get(link.borrow())?.text()?;
            let fragment = Html::parse_fragment(&xml);
            if let Ok(selector) = Selector::parse("script") {
                if let Some(element) = fragment.select(&selector).find(|el| {
                    el.attr("type") == Some("application/ld+json")
                        && el.inner_html().contains("recipeIngredient")
                }) {
                    let json: serde_json::Value =
                        serde_json::from_str(&element.text().collect::<Vec<&str>>().join(""))
                            .unwrap();
                    if json.is_object() {
                        json = json.get("@graph").unwrap().clone()
                        //TODO: change all unwrap to returns
                    }
                    json = json
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|&v| {
                            let otype = v.get("@type").unwrap();
                            println!("{:?}", otype);
                            match otype.as_array() {
                                Some(arr) => arr.contains(&json!("Recipe")),
                                None => otype.eq(&json!("Recipe")),
                            }
                        })
                        .unwrap()
                        .clone();
                    let image = get_image(&json);
                }
            }
        }
        (StatusCode::UNPROCESSABLE_ENTITY, Json(None))
    }
}

fn get_name(json: &Value) -> Option<String> {
    if let Some(name) = json.get("name") {
        if let Some(name) = name.as_str() {
            return Some(name.to_owned());
        }
    }
    return None;
}

fn get_cook_time(json: &Value) -> Option<i16> {
    if let Some(time) = json.get("totalTime") {
        if let Some(time) = time.as_str() {
            return iso8601_to_mins(time);
        }
    }
    return None;
}

fn get_instructions(json: &Value) -> Option<String> {
    todo!()
}

fn iso8601_to_mins(duration: &str) -> Option<u32> {
    if let Ok(duration) = iso8601::duration(duration) {
        if let iso8601::Duration::YMDHMS {
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
        } = duration
        {
            return Some(day * 60 * 60 + hour * 60 + minute + second / 60);
        }
    }
    return None;
}

fn get_image(json: &Value) -> Option<Url> {
    if let Some(image) = json.get("image") {
        match image {
            Value::String(s) => return Url::parse(s).ok(),
            Value::Array(a) => match a.get(0) {
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
    return None;
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
    return Vec::new();
}
