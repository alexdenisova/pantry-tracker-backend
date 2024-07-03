mod payload;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::database::errors::GetError;
use crate::server::routes::errors::{AppError, VerifyError};
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use payload::{
    CreatePayload, ListQueryParams, RecipeIngredientListResponse, RecipeIngredientResponse,
    UpdatePayload,
};

pub struct RecipeIngredientRouter {}

impl RecipeIngredientRouter {
    pub fn router() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(RecipeIngredientRouter::list)
                    .post(RecipeIngredientRouter::create),
            )
            .route(
                "/:id",
                get(RecipeIngredientRouter::get)
                    .put(RecipeIngredientRouter::update)
                    .delete(RecipeIngredientRouter::delete),
            )
    }

    async fn create(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<CreatePayload>,
    ) -> Result<(StatusCode, Json<RecipeIngredientResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                verify_user(&state, user_id, Some(payload.recipe_id)).await?;
                let recipe_ingredient = state
                    .db_client
                    .create_recipe_ingredient(payload.into())
                    .await?;
                log::info!(
                    "Recipe ingredient with id {:?} created",
                    recipe_ingredient.id.to_string()
                );
                return Ok((StatusCode::CREATED, Json(recipe_ingredient.into())));
            }
        }
        Err(AppError::Unauthorized)
    }

    pub async fn list(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> Result<(StatusCode, Json<RecipeIngredientListResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                verify_user(&state, user_id, query_params.recipe_id).await?;
                let recipe_ingredients = state
                    .db_client
                    .list_recipe_ingredients(query_params.into())
                    .await?;
                log::info!(
                    "{:?} recipe ingredients collected",
                    recipe_ingredients.items.len()
                );
                return Ok((StatusCode::OK, Json(recipe_ingredients.into())));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn get(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> Result<(StatusCode, Json<RecipeIngredientResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let recipe_ingredient = get_recipe_ingredient(&state, id).await?;
                verify_user(&state, user_id, Some(recipe_ingredient.recipe_id)).await?;
                return Ok((StatusCode::OK, Json(recipe_ingredient)));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn update(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
        jar: CookieJar,
        Json(payload): Json<UpdatePayload>,
    ) -> Result<(StatusCode, Json<RecipeIngredientResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let recipe_ingredient = get_recipe_ingredient(&state, id).await?;
                verify_user(&state, user_id, Some(recipe_ingredient.recipe_id)).await?;
                let recipe_ingredient = state
                    .db_client
                    .update_recipe_ingredient(id, payload.into())
                    .await?;
                log::info!("Updated recipe ingredient with id {id:?}");
                return Ok((StatusCode::OK, Json(recipe_ingredient.into())));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn delete(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> Result<StatusCode, AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let recipe_ingredient = get_recipe_ingredient(&state, id).await?;
                verify_user(&state, user_id, Some(recipe_ingredient.recipe_id)).await?;
                state.db_client.delete_recipe_ingredient(id).await?;
                log::info!("Deleted recipe ingredient with id {:?}", id);
                return Ok(StatusCode::NO_CONTENT);
            }
        }
        Err(AppError::Unauthorized)
    }
}

async fn verify_user(
    state: &AppState,
    user_id: Uuid,
    recipe_id: Option<Uuid>,
) -> Result<(), VerifyError> {
    if let Ok(true) = state.user_is_admin(user_id).await {
        return Ok(());
    }
    if let Some(recipe_id) = recipe_id {
        let recipe = state.db_client.get_recipe(recipe_id).await?;
        log::info!("Got recipe with id {:?}", recipe.id);
        if recipe.user_id == user_id {
            return Ok(());
        }
    }
    Err(VerifyError::Unauthorized)
}

async fn get_recipe_ingredient(
    state: &AppState,
    id: Uuid,
) -> Result<RecipeIngredientResponse, GetError> {
    state
        .db_client
        .get_recipe_ingredient(id)
        .await
        .map(Into::into)
}
