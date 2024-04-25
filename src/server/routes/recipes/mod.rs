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
use crate::server::routes::utils::VerifyError;
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
    ) -> (StatusCode, Json<Option<RecipeResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                match state
                    .db_client
                    .create_recipe(payload.into_dto(user_id))
                    .await
                {
                    Ok(recipe) => {
                        log::info!("Recipe with id {:?} created", recipe.id.to_string());
                        return (StatusCode::CREATED, Json(Some(recipe.into())));
                    }
                    Err(err) => {
                        if let CreateError::AlreadyExist { .. } = err {
                            log::error!("{}", err.to_string());
                            return (StatusCode::CONFLICT, Json(None));
                        }
                        log::error!("{}", err.to_string());
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                    }
                }
            }
        }
        log::debug!("Could not create recipe: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn list_recipes(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<RecipeListResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let ingredient_ids = query_params.ingredient_ids.clone();
                match state
                    .db_client
                    .list_recipes(query_params.into_dto(Some(user_id)))
                    .await
                {
                    Ok(recipes) => {
                        if let Some(ingredient_ids) = ingredient_ids {
                            return list_recipes_with_ingredients(state, ingredient_ids, recipes)
                                .await;
                        }
                        log::info!("{:?} recipes collected", recipes.items.len());
                        return (StatusCode::OK, Json(Some(recipes.into())));
                    }
                    Err(err) => {
                        log::error!("{}", err.to_string());
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                    }
                }
            }
        }
        log::debug!("Could not list recipes: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn get_recipe(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> (StatusCode, Json<Option<RecipeResponse>>) {
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
        log::debug!("Could not get recipe: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn update_recipe(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
        Json(payload): Json<UpdatePayload>,
    ) -> (StatusCode, Json<Option<RecipeResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if let Err(err) = verified_user(&state, id, user_id).await {
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
        log::debug!("Could not update recipe: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn delete_recipe(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> StatusCode {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if let Err(err) = verified_user(&state, id, user_id).await {
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
        log::debug!("Could not delete recipe: user unauthorized");
        StatusCode::UNAUTHORIZED
    }
}

async fn verified_user(state: &AppState, id: Uuid, user_id: Uuid) -> Result<(), VerifyError> {
    match state.db_client.get_recipe(id).await {
        Ok(recipe) => {
            if recipe.user_id == user_id {
                log::info!("Got recipe with id {:?}", recipe.id);
                return Ok(());
            }
            Err(VerifyError::Unauthorized)
        }
        Err(err) => {
            if let GetError::NotFound { .. } = err {
                log::error!("{}", err.to_string());
                return Err(VerifyError::NotFound);
            }
            log::error!("{}", err.to_string());
            Err(VerifyError::InternalServerError)
        }
    }
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
