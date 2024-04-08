use axum::extract::Query;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use payload::{CreatePayload, IngredientListResponse, IngredientResponse, UpdatePayload};

use crate::database::errors::{CreateError, DeleteError, GetError, UpdateError};
use crate::server::routes::ingredients::payload::ListQueryParams;
use crate::server::state::AppState;
use uuid::Uuid;

mod payload;

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
                    .put(IngredientRouter::update_ingredient)
                    .delete(IngredientRouter::delete_ingredient),
            )
    }

    async fn create_ingredient(
        State(state): State<AppState>,
        Json(payload): Json<CreatePayload>,
    ) -> (StatusCode, Json<Option<IngredientResponse>>) {
        match state.db_client.create_ingredient(payload.into()).await {
            Ok(ingredient) => {
                log::info!("Ingredient with id {:?} created", ingredient.id.to_string());
                (StatusCode::CREATED, Json(Some(ingredient.into())))
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

    async fn list_ingredients(
        State(state): State<AppState>,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<IngredientListResponse>>) {
        match state.db_client.list_ingredients(query_params.into()).await {
            Ok(ingredients) => {
                log::info!("{:?} ingredients collected", ingredients.items.len());
                (StatusCode::OK, Json(Some(ingredients.into())))
            }
            Err(err) => {
                log::error!("{}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
            }
        }
    }

    async fn get_ingredient(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
    ) -> (StatusCode, Json<Option<IngredientResponse>>) {
        match state.db_client.get_ingredient(id).await {
            Ok(ingredient) => {
                log::info!("Got ingredient with id {:?}", ingredient.id);
                (StatusCode::OK, Json(Some(ingredient.into())))
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

    async fn update_ingredient(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
        Json(payload): Json<UpdatePayload>,
    ) -> (StatusCode, Json<Option<IngredientResponse>>) {
        match state.db_client.update_ingredient(id, payload.into()).await {
            Ok(ingredient) => {
                log::info!("Updated ingredient with id {id:?}");
                (StatusCode::OK, Json(Some(ingredient.into())))
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

    async fn delete_ingredient(State(state): State<AppState>, Path(id): Path<Uuid>) -> StatusCode {
        match state.db_client.delete_ingredient(id).await {
            Ok(()) => {
                log::info!("Deleted ingredient with id {:?}", id);
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
