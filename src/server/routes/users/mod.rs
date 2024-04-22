mod payload;

use axum::extract::Query;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::database::errors::{CreateError, DeleteError, GetError, UpdateError};
use crate::server::routes::users::payload::ListQueryParams;
use crate::server::routes::COOKIE_KEY;
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
        jar: CookieJar,
        Json(mut payload): Json<CreatePayload>,
    ) -> (StatusCode, Json<Option<UserResponse>>) {
        if let Some(true) = payload.admin {
            if let Some(session_id) = jar.get(COOKIE_KEY) {
                if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await
                {
                    payload.admin = Some(state.user_is_admin(user_id).await.unwrap_or(false));
                }
            }
        };
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
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<UsersListResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if let Ok(true) = state.user_is_admin(user_id).await {
                    match state.db_client.list_users(query_params.into()).await {
                        Ok(users) => {
                            log::info!("{:?} users collected", users.items.len());
                            return (StatusCode::OK, Json(Some(users.into())));
                        }
                        Err(err) => {
                            log::error!("{}", err.to_string());
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                        }
                    }
                }
            }
        }
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn get_user(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> (StatusCode, Json<Option<UserResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if user_id == id || state.user_is_admin(user_id).await.unwrap_or(false) {
                    match state.db_client.get_user(id).await {
                        Ok(user) => {
                            log::info!("Got user with id {:?}", user.id);
                            return (StatusCode::OK, Json(Some(user.into())));
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
        }
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn update_user(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
        jar: CookieJar,
        Json(mut payload): Json<UpdatePayload>,
    ) -> (StatusCode, Json<Option<UserResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let is_admin = state.user_is_admin(user_id).await.unwrap_or(false);
                if user_id == id || is_admin {
                    payload.admin = Some(is_admin);
                    match state.db_client.update_user(id, payload.into()).await {
                        Ok(user) => {
                            log::info!("Updated user with id {id:?}");
                            return (StatusCode::OK, Json(Some(user.into())));
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
        }
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn delete_user(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
        jar: CookieJar,
    ) -> StatusCode {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if user_id == id || state.user_is_admin(user_id).await.unwrap_or(false) {
                    match state.db_client.delete_user(id).await {
                        Ok(()) => {
                            log::info!("Deleted user with id {:?}", id);
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
        }
        StatusCode::UNAUTHORIZED
    }
}
