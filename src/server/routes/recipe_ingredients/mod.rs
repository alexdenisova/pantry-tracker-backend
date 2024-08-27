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
    CreatePayload, ListQueryParams, RecipeIngredientJoinResponse, RecipeIngredientListResponse,
    RecipeIngredientResponse, UpdatePayload,
};

pub struct RecipeIngredientRouter {}

impl RecipeIngredientRouter {
    pub fn router() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(RecipeIngredientRouter::list).post(RecipeIngredientRouter::create),
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
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                verify_recipe_user(&state, payload.recipe_id, user_id).await?;
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
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                let user_id = if let Some(recipe_id) = query_params.recipe_id {
                    verify_recipe_user(&state, recipe_id, user_id).await?;
                    None
                } else {
                    Some(user_id)
                };
                let list_params = query_params.into_dto(user_id);
                let recipe_ingredients: Vec<RecipeIngredientJoinResponse> = state
                    .db_client
                    .list_recipe_ingredients(&list_params)
                    .await?
                    .into();
                log::info!(
                    "{:?} recipe ingredients collected",
                    recipe_ingredients.len()
                );
                let metadata = state
                    .db_client
                    .get_recipe_ingredients_metadata(&list_params)
                    .await?
                    .into();
                return Ok((
                    StatusCode::OK,
                    Json(RecipeIngredientListResponse::from(
                        recipe_ingredients,
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
    ) -> Result<(StatusCode, Json<RecipeIngredientResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                let recipe_ingredient = state.db_client.get_recipe_ingredient(id).await?;
                verify_recipe_user(&state, recipe_ingredient.recipe_id, user_id).await?;
                return Ok((StatusCode::OK, Json(recipe_ingredient.into())));
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
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                verify_user(&state, id, user_id).await?;
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
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                verify_user(&state, id, user_id).await?;
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
    recipe_ingredient_id: Uuid,
    user_id: Uuid,
) -> Result<(), VerifyError> {
    if state.user_is_admin(user_id).await? {
        return Ok(());
    }
    let recipe_ingredient = state
        .db_client
        .get_recipe_ingredient(recipe_ingredient_id)
        .await?;
    log::info!("Got recipe ingredient with id {:?}", recipe_ingredient.id);
    verify_recipe_user(state, recipe_ingredient.recipe_id, user_id).await
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
