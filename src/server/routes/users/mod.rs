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

use crate::server::routes::errors::AppError;
use crate::server::routes::users::payload::ListQueryParams;
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use payload::{CreatePayload, UpdatePayload, UserResponse, UsersListResponse};

pub struct UserRouter {}

impl UserRouter {
    pub fn router() -> Router<AppState> {
        Router::new()
            .route("/", get(UserRouter::list).post(UserRouter::create))
            .route(
                "/:id",
                get(UserRouter::get)
                    .put(UserRouter::update)
                    .delete(UserRouter::delete),
            )
    }

    async fn create(
        State(state): State<AppState>,
        Json(payload): Json<CreatePayload>,
    ) -> Result<(StatusCode, Json<UserResponse>), AppError> {
        let user = state.db_client.create_user(payload.into()).await?;
        log::info!("User with id {:?} created", user.id.to_string());
        Ok((StatusCode::CREATED, Json(user.into())))
    }

    async fn list(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> Result<(StatusCode, Json<UsersListResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                if state.user_is_admin(user_id).await? {
                    let list_params = query_params.into();
                    let users: Vec<UserResponse> =
                        state.db_client.list_users(&list_params).await?.into();
                    log::info!("{:?} users collected", users.len());
                    let metadata = state
                        .db_client
                        .get_users_metadata(&list_params)
                        .await?
                        .into();
                    return Ok((
                        StatusCode::OK,
                        Json(UsersListResponse::from(users, metadata)),
                    ));
                }
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn get(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> Result<(StatusCode, Json<UserResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                if user_id == id || state.user_is_admin(user_id).await? {
                    let user = state.db_client.get_user(id).await?;
                    log::info!("Got user with id {:?}", user.id);
                    return Ok((StatusCode::OK, Json(user.into())));
                }
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn update(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
        jar: CookieJar,
        Json(payload): Json<UpdatePayload>,
    ) -> Result<(StatusCode, Json<UserResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                if user_id == id || state.user_is_admin(user_id).await? {
                    let user = state.db_client.update_user(id, payload.into()).await?;
                    log::info!("Updated user with id {id:?}");
                    return Ok((StatusCode::OK, Json(user.into())));
                }
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn delete(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
        jar: CookieJar,
    ) -> Result<StatusCode, AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Some(user_id) = state.get_sessions_user(session_id.value_trimmed()).await? {
                if user_id == id || state.user_is_admin(user_id).await? {
                    state.db_client.delete_user(id).await?;
                    log::info!("Deleted user with id {:?}", id);
                    return Ok(StatusCode::NO_CONTENT);
                }
            }
        }
        Err(AppError::Unauthorized)
    }
}
