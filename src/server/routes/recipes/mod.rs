mod payload;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use color_eyre::Result as AnyResult;
use payload::{CreatePayload, ListQueryParams, RecipeListResponse, RecipeResponse, UpdatePayload};
use urlencoding::decode;

use crate::database::errors::{CreateError, DeleteError, GetError, UpdateError};
use crate::database::recipe_ingredients::dto::RecipeIngredientsListDto;
use crate::database::recipes::dto::RecipesListDto;
use crate::server::routes::errors::{AppError, VerifyError};
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use uuid::Uuid;

pub struct RecipeRouter {}

impl RecipeRouter {
    pub fn get() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(RecipeRouter::list_recipes).post(RecipeRouter::create_recipe),
            )
            .route(
                "/:id",
                get(RecipeRouter::get_recipe)
                    .put(RecipeRouter::update_recipe)
                    .delete(RecipeRouter::delete_recipe),
            )
    }

    async fn create_recipe(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<CreatePayload>,
    ) -> Result<(StatusCode, Json<RecipeResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let recipe = state
                    .db_client
                    .create_recipe(payload.into_dto(user_id))
                    .await?;
                log::info!("Recipe with id {:?} created", recipe.id.to_string());
                return Ok((StatusCode::CREATED, Json(recipe.into())));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn list_recipes(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> Result<(StatusCode, Json<RecipeListResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let ingredient_ids = query_params.ingredient_ids.clone();
                let recipes = state
                    .db_client
                    .list_recipes(query_params.into_dto(Some(user_id)))
                    .await?;
                if let Some(ingredient_ids) = ingredient_ids {
                    return list_recipes_with_ingredients(state, ingredient_ids, recipes).await;
                }
                log::info!("{:?} recipes collected", recipes.items.len());
                return Ok((StatusCode::OK, Json(recipes.into())));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn get_recipe(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> Result<(StatusCode, Json<RecipeResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                match state.db_client.get_recipe(id).await {
                    Ok(recipe) => {
                        if recipe.user_id == user_id {
                            log::info!("Got recipe with id {:?}", recipe.id);
                            return (StatusCode::OK, Json(Some(recipe.into())));
                        }
                    }
                    Err(err) => {
                        if let GetError::NotFound { .. } = err {
                            log::error!("{}", err.to_string());
                            return (StatusCode::NOT_FOUND, Json(None));
                        }
                        log::error!("{}", err.to_string());
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                    }
                }
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn update_recipe(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
        Json(payload): Json<UpdatePayload>,
    ) -> Result<(StatusCode, Json<RecipeResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if let Err(err) = verify_user(&state, id, user_id).await {
                    return (err.into(), Json(None));
                }
                match state
                    .db_client
                    .update_recipe(id, payload.into_dto(user_id))
                    .await
                {
                    Ok(recipe) => {
                        log::info!("Updated recipe with id {id:?}");
                        return (StatusCode::OK, Json(Some(recipe.into())));
                    }
                    Err(err) => {
                        if let UpdateError::NotFound { .. } = err {
                            log::error!("{}", err.to_string());
                            return (StatusCode::NOT_FOUND, Json(None));
                        }
                        log::error!("{}", err.to_string());
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                    }
                }
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn delete_recipe(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> Result<StatusCode, AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if let Err(err) = verify_user(&state, id, user_id).await {
                    return err.into();
                }
                match state.db_client.delete_recipe(id).await {
                    Ok(()) => {
                        log::info!("Deleted recipe with id {:?}", id);
                        return StatusCode::NO_CONTENT;
                    }
                    Err(err) => {
                        if let DeleteError::NotFound { .. } = err {
                            log::error!("{}", err.to_string());
                            return StatusCode::NOT_FOUND;
                        }
                        log::error!("{}", err.to_string());
                        return StatusCode::INTERNAL_SERVER_ERROR;
                    }
                }
            }
        }
        Err(AppError::Unauthorized)
    }
}

async fn verify_user(state: &AppState, user_id: Uuid, recipe_id: Uuid) -> Result<(), VerifyError> {
    let recipe = state.db_client.get_recipe(recipe_id).await?;
    if recipe.user_id == user_id {
        log::info!("Got recipe with id {:?}", recipe.id);
        return Ok(());
    }
    Err(VerifyError::Unauthorized)
}

async fn list_recipes_with_ingredients(
    state: AppState,
    ingredient_ids: String,
    recipes: RecipesListDto,
) -> (StatusCode, Json<Option<RecipeListResponse>>) {
    if let Ok(ingredient_ids) = decode(&ingredient_ids) {
        if let Ok(ingredient_ids) = serde_json::from_str::<Vec<Uuid>>(&ingredient_ids) {
            let mut recipes_with_ingredients: Vec<RecipeResponse> = Vec::new();

            'outer: for recipe in recipes.items {
                let Ok(recipe_ingredients) = get_recipe_ingredients(&state, recipe.id).await else {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                };

                for id in &ingredient_ids {
                    if !recipe_ingredients
                        .items
                        .iter()
                        .any(|ingredient| ingredient.ingredient_id.eq(id))
                    {
                        continue 'outer;
                    }
                }
                recipes_with_ingredients.push(recipe.into());
            }
            log::info!(
                "{:?} recipes with ingredients collected",
                recipes_with_ingredients.len()
            );
            return (
                StatusCode::OK,
                Json(Some(RecipeListResponse {
                    items: recipes_with_ingredients,
                })),
            );
        }
    }
    (StatusCode::UNPROCESSABLE_ENTITY, Json(None))
}

async fn get_recipe_ingredients(
    state: &AppState,
    recipe_id: Uuid,
) -> AnyResult<RecipeIngredientsListDto> {
    match state
        .db_client
        .list_recipe_ingredients(crate::database::recipe_ingredients::dto::ListParamsDto {
            recipe_id: Some(recipe_id),
            ingredient_id: None,
        })
        .await
    {
        Ok(recipe_ingredients) => {
            log::info!("{:?} recipes collected", recipe_ingredients.items.len());
            Ok(recipe_ingredients)
        }
        Err(err) => {
            log::error!("{}", err.to_string());
            Err(err.into())
        }
    }
}
