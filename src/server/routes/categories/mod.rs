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
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use payload::{CreatePayload, CategoryListResponse, CategoryResponse, ListQueryParams};

pub struct CategoryRouter {}

impl CategoryRouter {
    pub fn router() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(CategoryRouter::list).post(CategoryRouter::create),
            )
            .route(
                "/:id",
                get(CategoryRouter::get).delete(CategoryRouter::delete),
            )
    }

    async fn create(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<CreatePayload>,
    ) -> Result<(StatusCode, Json<CategoryResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if state.session_is_valid(session_id.value_trimmed()).await? {
                let category = state.db_client.create_category(payload.into()).await?;
                log::info!("Category with id {:?} created", category.id.to_string());
                return Ok((StatusCode::CREATED, Json(category.into())));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn list(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> Result<(StatusCode, Json<CategoryListResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if state.session_is_valid(session_id.value_trimmed()).await? {
                let list_params = query_params.into();
                let categories: Vec<CategoryResponse> =
                    state.db_client.list_categories(&list_params).await?.into();
                log::info!("{:?} categories collected", categories.len());
                let metadata = state
                    .db_client
                    .get_categories_metadata(&list_params)
                    .await?
                    .into();
                return Ok((
                    StatusCode::OK,
                    Json(CategoryListResponse::from(categories, metadata)),
                ));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn get(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> Result<(StatusCode, Json<CategoryResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if state.session_is_valid(session_id.value_trimmed()).await? {
                let category = state.db_client.get_category(id).await?;
                log::info!("Got category with id {:?}", category.id);
                return Ok((StatusCode::OK, Json(category.into())));
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
                if state.user_is_admin(user_id).await? {
                    state.db_client.delete_category(id).await?;
                    log::info!("Deleted category with id {:?}", id);
                    return Ok(StatusCode::NO_CONTENT);
                }
            }
        }
        Err(AppError::Unauthorized)
    }
}
