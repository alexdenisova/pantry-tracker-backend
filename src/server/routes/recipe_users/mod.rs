mod payload;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use uuid::Uuid;

use crate::database::errors::{CreateError, DeleteError, GetError};
use crate::server::state::AppState;
use payload::{CreatePayload, ListQueryParams, RecipeUserListResponse, RecipeUserResponse};

pub struct RecipeUserRouter {}

impl RecipeUserRouter {
    pub fn get() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(RecipeUserRouter::list_recipe_users).post(RecipeUserRouter::create_recipe_user),
            )
            .route(
                "/:id",
                get(RecipeUserRouter::get_recipe_user).delete(RecipeUserRouter::delete_recipe_user),
            )
    }

    async fn create_recipe_user(
        State(state): State<AppState>,
        Json(payload): Json<CreatePayload>,
    ) -> (StatusCode, Json<Option<RecipeUserResponse>>) {
        match state.db_client.create_recipe_user(payload.into()).await {
            Ok(recipe_user) => {
                log::info!(
                    "Recipe user with id {:?} created",
                    recipe_user.id.to_string()
                );
                (StatusCode::CREATED, Json(Some(recipe_user.into())))
            }
            Err(err) => {
                if let CreateError::AlreadyExist { .. } = err {
                    log::error!("{}", err.to_string());
                    (StatusCode::CONFLICT, Json(None))
                } else {
                    log::error!("{}", err.to_string());
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
                }
            }
        }
    }

    async fn list_recipe_users(
        State(state): State<AppState>,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<RecipeUserListResponse>>) {
        match state.db_client.list_recipe_users(query_params.into()).await {
            Ok(recipe_users) => {
                log::info!("{:?} recipe users collected", recipe_users.items.len());
                (StatusCode::OK, Json(Some(recipe_users.into())))
            }
            Err(err) => {
                log::error!("{}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
            }
        }
    }

    async fn get_recipe_user(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
    ) -> (StatusCode, Json<Option<RecipeUserResponse>>) {
        match state.db_client.get_recipe_user(id).await {
            Ok(recipe_user) => {
                log::info!("Got recipe user with id {:?}", recipe_user.id);
                (StatusCode::OK, Json(Some(recipe_user.into())))
            }
            Err(err) => {
                if let GetError::NotFound { .. } = err {
                    log::error!("{}", err.to_string());
                    (StatusCode::NOT_FOUND, Json(None))
                } else {
                    log::error!("{}", err.to_string());
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
                }
            }
        }
    }

    async fn delete_recipe_user(State(state): State<AppState>, Path(id): Path<Uuid>) -> StatusCode {
        match state.db_client.delete_recipe_user(id).await {
            Ok(()) => {
                log::info!("Deleted recipe user with id {:?}", id);
                StatusCode::NO_CONTENT
            }
            Err(err) => {
                if let DeleteError::NotFound { .. } = err {
                    log::error!("{}", err.to_string());
                    StatusCode::NOT_FOUND
                } else {
                    log::error!("{}", err.to_string());
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
    }
}
