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
use payload::{
    CreatePayload, IngredientListResponse, IngredientResponse, ListQueryParams,
};

pub struct IngredientRouter {}

impl IngredientRouter {
    pub fn get() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(IngredientRouter::list_ingredients).post(IngredientRouter::create_ingredient),
            )
            .route(
                "/:id",
                get(IngredientRouter::get_ingredient)
                    .delete(IngredientRouter::delete_ingredient),
            )
    }

    async fn create_ingredient(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<CreatePayload>,
    ) -> (StatusCode, Json<Option<IngredientResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(true) = state.session_is_valid(session_id.value_trimmed()).await {
                match state.db_client.create_ingredient(payload.into()).await {
                    Ok(ingredient) => {
                        log::info!("Ingredient with id {:?} created", ingredient.id.to_string());
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
        log::debug!("Could not create ingredient: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn list_ingredients(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<IngredientListResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(true) = state.session_is_valid(session_id.value_trimmed()).await {
                match state.db_client.list_ingredients(query_params.into()).await {
                    Ok(ingredients) => {
                        log::info!("{:?} ingredients collected", ingredients.items.len());
                        return (StatusCode::OK, Json(Some(ingredients.into())));
                    }
                    Err(err) => {
                        log::error!("{}", err.to_string());
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                    }
                }
            }
        }
        log::debug!("Could not list ingredients: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn get_ingredient(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> (StatusCode, Json<Option<IngredientResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(true) = state.session_is_valid(session_id.value_trimmed()).await {
                match state.db_client.get_ingredient(id).await {
                    Ok(ingredient) => {
                        log::info!("Got ingredient with id {:?}", ingredient.id);
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
        log::debug!("Could not get ingredient: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn delete_ingredient(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> StatusCode {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if state.user_is_admin(user_id).await.unwrap_or(false) {
                    match state.db_client.delete_ingredient(id).await {
                        Ok(()) => {
                            log::info!("Deleted ingredient with id {:?}", id);
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
        log::debug!("Could not delete ingredient: user unauthorized");
        StatusCode::UNAUTHORIZED
    }
}
