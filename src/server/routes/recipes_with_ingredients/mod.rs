use crate::{
    database::recipe_ingredients::dto::ListParamsDto as RecipeIngredientListDto, server::AppState,
};
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use uuid::Uuid;

use self::payload::{ListPayload, ListQueryParams, RecipesWithIngredientsResponse};

mod payload;

pub struct RecipesWithIngredientsRouter {}

impl RecipesWithIngredientsRouter {
    pub fn list() -> Router<AppState> {
        Router::new().route(
            "/",
            get(RecipesWithIngredientsRouter::list_possible_recipes),
        )
    }

    async fn list_possible_recipes(
        State(state): State<AppState>,
        Query(query_params): Query<ListQueryParams>,
        Json(payload): Json<ListPayload>,
    ) -> (StatusCode, Json<Option<RecipesWithIngredientsResponse>>) {
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
        let mut recipe_ids: Vec<Vec<Uuid>> = Vec::with_capacity(payload.ingredient_ids.len() - 1); // ordering the recipes by the amount of desired ingredients found in them
        for recipe in &recipes.items {
            let recipe_ingredients = match state
                .db_client
                .list_recipe_ingredients(RecipeIngredientListDto {
                    recipe_id: Some(recipe.recipe_id.clone()),
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
            let mut number_of_found_ingredients = 0;
            for ingredient in recipe_ingredients.items {
                if payload
                    .ingredient_ids
                    .iter()
                    .any(|id| ingredient.ingredient_id.eq(id))
                {
                    number_of_found_ingredients += 1;
                }
            }
            if number_of_found_ingredients != 0 {
                recipe_ids[number_of_found_ingredients - 1].push(recipe.id);
            }
        }
        let mut response = RecipesWithIngredientsResponse {
            recipe_ids: Vec::new(),
        };
        for i in 0..recipe_ids.len() {
            for id in &recipe_ids[recipe_ids.len() - i - 1] {
                response.recipe_ids.push(id.clone());
            }
        }
        return (StatusCode::OK, Json(Some(response)));
    }
}
