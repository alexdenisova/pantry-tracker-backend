mod payload;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::server::routes::errors::{AppError, VerifyError};
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use payload::{
    CreatePayload, ListQueryParams, RecipeCategoryJoinResponse, RecipeCategoryListResponse,
    RecipeCategoryResponse,
};

pub struct RecipeCategoryRouter {}

impl RecipeCategoryRouter {
    pub fn router() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(RecipeCategoryRouter::list).post(RecipeCategoryRouter::create),
            )
            .route(
                "/:id",
                get(RecipeCategoryRouter::get).delete(RecipeCategoryRouter::delete),
            )
    }

    async fn create(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<CreatePayload>,
    ) -> Result<(StatusCode, Json<RecipeCategoryResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                verify_recipe_user(&state, payload.recipe_id, user_id).await?;
                let recipe_category = state
                    .db_client
                    .create_recipe_category(payload.into())
                    .await?;
                log::info!(
                    "Recipe category with id {:?} created",
                    recipe_category.id.to_string()
                );
                return Ok((StatusCode::CREATED, Json(recipe_category.into())));
            }
        }
        Err(AppError::Unauthorized)
    }

    pub async fn list(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> Result<(StatusCode, Json<RecipeCategoryListResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                let list_params = query_params.into_dto(user_id);
                let recipe_categories: Vec<RecipeCategoryJoinResponse> = state
                    .db_client
                    .list_recipe_categories(&list_params)
                    .await?
                    .into();
                log::info!("{:?} recipe categories collected", recipe_categories.len());
                let metadata = state
                    .db_client
                    .get_recipe_categories_metadata(&list_params)
                    .await?
                    .into();
                return Ok((
                    StatusCode::OK,
                    Json(RecipeCategoryListResponse::from(
                        recipe_categories,
                        metadata,
                    )),
                ));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn get(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> Result<(StatusCode, Json<RecipeCategoryResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                let recipe_category = state.db_client.get_recipe_category(id).await?;
                if recipe_category.user_id == user_id {
                    return Ok((StatusCode::OK, Json(recipe_category.into())));
                }
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
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                let recipe_category = state.db_client.get_recipe_category(id).await?;
                if recipe_category.user_id == user_id {
                    state.db_client.delete_recipe_category(id).await?;
                    log::info!("Deleted recipe category with id {:?}", id);
                    return Ok(StatusCode::NO_CONTENT);
                }
            }
        }
        Err(AppError::Unauthorized)
    }
}

async fn verify_recipe_user(
    state: &AppState,
    recipe_id: Uuid,
    user_id: Uuid,
) -> Result<(), VerifyError> {
    let recipe = state.db_client.get_recipe(recipe_id).await?;
    log::info!("Got recipe with id {:?}", recipe.id);
    if recipe.user_id == user_id {
        return Ok(());
    }
    Err(VerifyError::Unauthorized)
}
