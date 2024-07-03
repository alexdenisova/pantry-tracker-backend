mod payload;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use color_eyre::Result as AnyResult;
use uuid::Uuid;

use self::payload::PossibleRecipesResponse;
use crate::database::pantry_items::dto::{
    ListParamsDto as PantryItemsListParamsDto, PantryItemsListDto,
};
use crate::database::recipes::dto::ListParamsDto as RecipesListParamsDto;
use crate::database::recipes::dto::RecipesListDto;
use crate::server::routes::COOKIE_KEY;
use crate::{
    database::recipe_ingredients::dto::ListParamsDto as RecipeIngredientListDto, server::AppState,
};

pub struct PossibleRecipesRouter {}

impl PossibleRecipesRouter {
    pub fn list() -> Router<AppState> {
        Router::new().route("/", get(PossibleRecipesRouter::list_possible_recipes))
    }

    async fn list_possible_recipes(
        State(state): State<AppState>,
        jar: CookieJar,
    ) -> (StatusCode, Json<Option<PossibleRecipesResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let Ok(pantry_items) = list_pantry_items(&state, user_id).await else {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                };
                let Ok(recipes) = list_recipes(&state, user_id).await else {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                };
                let mut possbile_recipe_ids = Vec::new();
                'outer: for recipe in &recipes.items {
                    let recipe_ingredients = match state
                        .db_client
                        .list_recipe_ingredients(RecipeIngredientListDto {
                            recipe_id: Some(recipe.id),
                        })
                        .await
                    {
                        Ok(recipe_ingredients) => {
                            log::info!(
                                "{:?} recipe ingredients collected",
                                recipe_ingredients.items.len()
                            );
                            recipe_ingredients
                        }
                        Err(err) => {
                            log::error!("{}", err.to_string());
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                        }
                    };
                    for ingredient in recipe_ingredients.items {
                        if ingredient.optional {
                            continue;
                        }
                        if !pantry_items
                            .items
                            .iter()
                            .any(|x| x.ingredient_id == ingredient.ingredient_id)
                        {
                            continue 'outer;
                        }
                    }
                    possbile_recipe_ids.push(recipe.id);
                }
                return (
                    StatusCode::OK,
                    Json(Some(PossibleRecipesResponse {
                        recipe_ids: possbile_recipe_ids,
                    })),
                );
            }
        }
        log::debug!("Could not list possible recipes: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }
}

async fn list_pantry_items(state: &AppState, user_id: Uuid) -> AnyResult<PantryItemsListDto> {
    match state
        .db_client
        .list_pantry_items(PantryItemsListParamsDto {
            max_expiration_date: None,
            user_id: Some(user_id),
            ingredient_id: None,
        })
        .await
    {
        Ok(pantry_items) => {
            log::info!("{:?} pantry items collected", pantry_items.items.len());
            Ok(pantry_items)
        }
        Err(err) => {
            log::error!("{}", err.to_string());
            Err(eyre!("Internal server error"))
        }
    }
}

async fn list_recipes(state: &AppState, user_id: Uuid) -> AnyResult<RecipesListDto> {
    match state
        .db_client
        .list_recipes(RecipesListParamsDto {
            user_id: Some(user_id),
            name_contains: None,
            total_time_mins: None,
        })
        .await
    {
        Ok(res) => {
            log::info!("{:?} recipes collected", res.items.len());
            Ok(res)
        }
        Err(err) => {
            log::error!("{}", err.to_string());
            Err(eyre!("Internal server error"))
        }
    }
}
