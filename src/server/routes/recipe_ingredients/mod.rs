mod payload;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::database::errors::{CreateError, DeleteError, GetError, UpdateError};
use crate::server::routes::utils::VerifyError;
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use payload::{
    CreatePayload, ListQueryParams, RecipeIngredientListResponse, RecipeIngredientResponse,
    UpdatePayload,
};

pub struct RecipeIngredientRouter {}

impl RecipeIngredientRouter {
    pub fn get() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(RecipeIngredientRouter::list_recipe_ingredients)
                    .post(RecipeIngredientRouter::create_recipe_ingredient),
            )
            .route(
                "/:id",
                get(RecipeIngredientRouter::get_recipe_ingredient)
                    .put(RecipeIngredientRouter::update_recipe_ingredient)
                    .delete(RecipeIngredientRouter::delete_recipe_ingredient),
            )
    }

    async fn create_recipe_ingredient(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<CreatePayload>,
    ) -> (StatusCode, Json<Option<RecipeIngredientResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if let Err(err) = verified_user(&state, payload.recipe_id, user_id).await {
                    return (err.into(), Json(None));
                }
                match state
                    .db_client
                    .create_recipe_ingredient(payload.into())
                    .await
                {
                    Ok(recipe_ingredient) => {
                        log::info!(
                            "Recipe ingredient with id {:?} created",
                            recipe_ingredient.id.to_string()
                        );
                        return (StatusCode::CREATED, Json(Some(recipe_ingredient.into())));
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
        log::debug!("Could not create recipe ingredient: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    pub async fn list_recipe_ingredients(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<RecipeIngredientListResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if let Err(err) = verified_user(&state, query_params.recipe_id, user_id).await {
                    return (err.into(), Json(None));
                }
                match state
                    .db_client
                    .list_recipe_ingredients(query_params.into())
                    .await
                {
                    Ok(recipe_ingredients) => {
                        log::info!(
                            "{:?} recipe ingredients collected",
                            recipe_ingredients.items.len()
                        );
                        return (StatusCode::OK, Json(Some(recipe_ingredients.into())));
                    }
                    Err(err) => {
                        log::error!("{}", err.to_string());
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                    }
                }
            }
        }
        log::debug!("Could not list recipe ingredients: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn get_recipe_ingredient(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> (StatusCode, Json<Option<RecipeIngredientResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                return get_recipe_ingredient(&state, user_id, id).await;
            }
        }
        log::debug!("Could not get recipe ingredient: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn update_recipe_ingredient(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
        jar: CookieJar,
        Json(payload): Json<UpdatePayload>,
    ) -> (StatusCode, Json<Option<RecipeIngredientResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let (status, body) = get_recipe_ingredient(&state, user_id, id).await;
                if !status.is_success() {
                    return (status, body);
                }
                match state
                    .db_client
                    .update_recipe_ingredient(id, payload.into())
                    .await
                {
                    Ok(recipe_ingredient) => {
                        log::info!("Updated recipe ingredient with id {id:?}");
                        return (StatusCode::OK, Json(Some(recipe_ingredient.into())));
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
        log::debug!("Could not update recipe ingredient: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn delete_recipe_ingredient(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> StatusCode {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let (status, _) = get_recipe_ingredient(&state, user_id, id).await;
                if !status.is_success() {
                    return status;
                }
                match state.db_client.delete_recipe_ingredient(id).await {
                    Ok(()) => {
                        log::info!("Deleted recipe ingredient with id {:?}", id);
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
        log::debug!("Could not update recipe ingredient: user unauthorized");
        StatusCode::UNAUTHORIZED
    }
}

async fn verified_user(
    state: &AppState,
    recipe_id: Uuid,
    user_id: Uuid,
) -> Result<(), VerifyError> {
    match state.db_client.get_recipe(recipe_id).await {
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

async fn get_recipe_ingredient(
    state: &AppState,
    user_id: Uuid,
    id: Uuid,
) -> (StatusCode, Json<Option<RecipeIngredientResponse>>) {
    match state.db_client.get_recipe_ingredient(id).await {
        Ok(recipe_ingredient) => {
            if let Err(err) = verified_user(state, recipe_ingredient.recipe_id, user_id).await {
                return (err.into(), Json(None));
            }
            log::info!("Got recipe ingredient with id {:?}", recipe_ingredient.id);
            (StatusCode::OK, Json(Some(recipe_ingredient.into())))
        }
        Err(err) => {
            if let GetError::NotFound { .. } = err {
                log::error!("{}", err.to_string());
                return (StatusCode::NOT_FOUND, Json(None));
            }
            log::error!("{}", err.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
        }
    }
}
