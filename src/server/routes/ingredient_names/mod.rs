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

use crate::database::errors::{CreateError, DeleteError, GetError};
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use payload::{CreatePayload, IngredientNameListResponse, IngredientNameResponse, ListQueryParams};

pub struct IngredientNameRouter {}

impl IngredientNameRouter {
    pub fn get() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(IngredientNameRouter::list_ingredient_names).post(IngredientNameRouter::create_ingredient_name),
            )
            .route(
                "/:id",
                get(IngredientNameRouter::get_ingredient_name).delete(IngredientNameRouter::delete_ingredient_name),
            )
    }

    async fn create_ingredient_name(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<CreatePayload>,
    ) -> (StatusCode, Json<Option<IngredientNameResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(true) = state.session_is_valid(session_id.value_trimmed()).await {
                match state.db_client.create_ingredient_name(payload.into()).await {
                    Ok(ingredient) => {
                        log::info!("Ingredient name with id {:?} created", ingredient.id.to_string());
                        return (StatusCode::CREATED, Json(Some(ingredient.into())));
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
        log::debug!("Could not create ingredient name: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn list_ingredient_names(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<IngredientNameListResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(true) = state.session_is_valid(session_id.value_trimmed()).await {
                match state.db_client.list_ingredient_names(query_params.into()).await {
                    Ok(ingredients) => {
                        log::info!("{:?} ingredient names collected", ingredients.items.len());
                        return (StatusCode::OK, Json(Some(ingredients.into())));
                    }
                    Err(err) => {
                        log::error!("{}", err.to_string());
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                    }
                }
            }
        }
        log::debug!("Could not list ingredient names: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn get_ingredient_name(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> (StatusCode, Json<Option<IngredientNameResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(true) = state.session_is_valid(session_id.value_trimmed()).await {
                match state.db_client.get_ingredient_name(id).await {
                    Ok(ingredient) => {
                        log::info!("Got ingredient name with id {:?}", ingredient.id);
                        return (StatusCode::OK, Json(Some(ingredient.into())));
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
        log::debug!("Could not get ingredient name: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn delete_ingredient_name(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> StatusCode {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if state.user_is_admin(user_id).await.unwrap_or(false) {
                    match state.db_client.delete_ingredient_name(id).await {
                        Ok(()) => {
                            log::info!("Deleted ingredient name with id {:?}", id);
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
        log::debug!("Could not delete ingredient name: user unauthorized");
        StatusCode::UNAUTHORIZED
    }
}
