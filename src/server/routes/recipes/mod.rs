mod payload;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use payload::{CreatePayload, ListQueryParams, RecipeListResponse, RecipeResponse, UpdatePayload};
use urlencoding::decode;

use crate::database::errors::ListError;
use crate::server::routes::errors::{AppError, VerifyError};
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use uuid::Uuid;

pub struct RecipeRouter {}

impl RecipeRouter {
    pub fn router() -> Router<AppState> {
        Router::new()
            .route("/", get(RecipeRouter::list).post(RecipeRouter::create))
            .route(
                "/:id",
                get(RecipeRouter::get)
                    .put(RecipeRouter::update)
                    .delete(RecipeRouter::delete),
            )
    }

    async fn create(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<CreatePayload>,
    ) -> Result<(StatusCode, Json<RecipeResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
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

    async fn list(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> Result<(StatusCode, Json<RecipeListResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                let ingredient_ids = query_params.ingredient_ids.clone();
                let recipes = if ingredient_ids.is_some() {
                    list_recipes_containing_ingredients(state, user_id, query_params).await?
                } else {
                    let list_params = query_params.into_dto(user_id);
                    let recipes = state.db_client.list_recipes(&list_params).await?.into();
                    let metadata = state
                        .db_client
                        .get_recipes_metadata(&list_params)
                        .await?
                        .into();
                    RecipeListResponse::from(recipes, metadata)
                };
                log::info!("{:?} recipes collected", recipes.items.len());
                return Ok((StatusCode::OK, Json(recipes)));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn get(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> Result<(StatusCode, Json<RecipeResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                let recipe = state.db_client.get_recipe(id).await?;
                if recipe.user_id == user_id {
                    log::info!("Got recipe with id {:?}", recipe.id);
                    return Ok((StatusCode::OK, Json(recipe.into())));
                }
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn update(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
        Json(payload): Json<UpdatePayload>,
    ) -> Result<(StatusCode, Json<RecipeResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                verify_user(&state, id, user_id).await?;
                let recipe = state
                    .db_client
                    .update_recipe(id, payload.into_dto(user_id))
                    .await?;
                log::info!("Updated recipe with id {id:?}");
                return Ok((StatusCode::OK, Json(recipe.into())));
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
                state.db_client.delete_recipe(id).await?;
                log::info!("Deleted recipe with id {:?}", id);
                return Ok(StatusCode::NO_CONTENT);
            }
        }
        Err(AppError::Unauthorized)
    }
}

async fn verify_user(state: &AppState, recipe_id: Uuid, user_id: Uuid) -> Result<(), VerifyError> {
    let recipe = state.db_client.get_recipe(recipe_id).await?;
    if recipe.user_id == user_id {
        log::info!("Got recipe with id {:?}", recipe.id);
        return Ok(());
    }
    Err(VerifyError::Unauthorized)
}

async fn list_recipes_containing_ingredients(
    state: AppState,
    user_id: Uuid,
    query_params: ListQueryParams,
) -> Result<RecipeListResponse, ListError> {
    if let Ok(ingredient_ids) = decode(&query_params.ingredient_ids.clone().unwrap()) {
        if let Ok(ingredient_ids) = serde_json::from_str::<Vec<Uuid>>(&ingredient_ids) {
            let list_params = query_params.into_join_dto(user_id, ingredient_ids);
            let recipes: Vec<RecipeResponse> = state
                .db_client
                .list_recipes_join(&list_params)
                .await?
                .into_iter()
                .map(Into::into)
                .collect();

            let metadata = state
                .db_client
                .get_recipes_join_metadata(&list_params)
                .await?
                .into();
            return Ok(RecipeListResponse::from(recipes, metadata));
        }
    }
    Err(ListError::Unprocessable {
        error: eyre!("ingredient_ids must be list of uuids seperated by commas."),
    })
}
