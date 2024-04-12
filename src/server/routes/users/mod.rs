mod payload;

use axum::extract::Query;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use uuid::Uuid;

use crate::database::errors::{CreateError, DeleteError, GetError, UpdateError};
use crate::server::routes::users::payload::ListQueryParams;
use crate::server::state::AppState;
use payload::{CreatePayload, UpdatePayload, UserResponse, UsersListResponse};

pub struct UserRouter {}

impl UserRouter {
    pub fn get() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(UserRouter::list_users).post(UserRouter::create_user),
            )
            .route(
                "/:id",
                get(UserRouter::get_user)
                    .put(UserRouter::update_user)
                    .delete(UserRouter::delete_user),
            )
    }

    async fn create_user(
        State(state): State<AppState>,
        Json(payload): Json<CreatePayload>,
    ) -> (StatusCode, Json<Option<UserResponse>>) {
        match state.db_client.create_user(payload.into()).await {
            Ok(user) => {
                log::info!("User with id {:?} created", user.id.to_string());
                (StatusCode::CREATED, Json(Some(user.into())))
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

    async fn list_users(
        State(state): State<AppState>,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<UsersListResponse>>) {
        match state.db_client.list_users(query_params.into()).await {
            Ok(users) => {
                log::info!("{:?} users collected", users.items.len());
                (StatusCode::OK, Json(Some(users.into())))
            }
            Err(err) => {
                log::error!("{}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
            }
        }
    }

    async fn get_user(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
    ) -> (StatusCode, Json<Option<UserResponse>>) {
        match state.db_client.get_user(id).await {
            Ok(user) => {
                log::info!("Got user with id {:?}", user.id);
                (StatusCode::OK, Json(Some(user.into())))
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

    async fn update_user(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
        Json(payload): Json<UpdatePayload>,
    ) -> (StatusCode, Json<Option<UserResponse>>) {
        match state.db_client.update_user(id, payload.into()).await {
            Ok(user) => {
                log::info!("Updated user with id {id:?}");
                (StatusCode::OK, Json(Some(user.into())))
            }
            Err(err) => {
                if let UpdateError::NotFound { .. } = err {
                    log::error!("{}", err.to_string());
                    (StatusCode::NOT_FOUND, Json(None))
                } else {
                    log::error!("{}", err.to_string());
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
                }
            }
        }
    }

    async fn delete_user(State(state): State<AppState>, Path(id): Path<Uuid>) -> StatusCode {
        match state.db_client.delete_user(id).await {
            Ok(()) => {
                log::info!("Deleted user with id {:?}", id);
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
