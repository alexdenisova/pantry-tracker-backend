use crate::{
    database::recipe_ingredients::dto::ListParamsDto as RecipeIngredientListDto, server::AppState,
};
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};

use self::payload::{ListQueryParams, PossibleRecipesResponse};

mod payload;

pub struct PossibleRecipesRouter {}

impl PossibleRecipesRouter {
    pub fn list() -> Router<AppState> {
        Router::new().route("/", get(PossibleRecipesRouter::list_possible_recipes))
    }

    async fn list_possible_recipes(
        State(state): State<AppState>,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<PossibleRecipesResponse>>) {
        let pantry_items = match state
            .db_client
            .list_pantry_items(query_params.clone().into())
            .await
        {
            Ok(pantry_items) => {
                log::info!("{:?} pantry items collected", pantry_items.items.len());
                pantry_items
            }
            Err(err) => {
                log::error!("{}", err.to_string());
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
            }
        };
        let recipes = match state.db_client.list_recipe_users(query_params.into()).await {
            Ok(recipe_users) => {
                log::info!("{:?} recipe users collected", recipe_users.items.len());
                recipe_users
            }
            Err(err) => {
                log::error!("{}", err.to_string());
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
            }
        };
        let mut possbile_recipe_ids = Vec::new();
        'outer: for recipe in &recipes.items {
            let recipe_ingredients = match state
                .db_client
                .list_recipe_ingredients(RecipeIngredientListDto {
                    recipe_id: Some(recipe.recipe_id),
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
        (
            StatusCode::OK,
            Json(Some(PossibleRecipesResponse {
                recipe_ids: possbile_recipe_ids,
            })),
        )
    }
}
