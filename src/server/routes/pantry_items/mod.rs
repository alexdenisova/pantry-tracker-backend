use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use payload::{
    CreatePayload, PantryItemListResponse, PantryItemResponse, ListQueryParams, UpdatePayload,
};

use crate::database::errors::{CreateError, DeleteError, GetError, UpdateError};
use crate::server::state::AppState;
use uuid::Uuid;

mod payload;

pub struct PantryItemRouter {}

impl PantryItemRouter {
    pub fn get() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(PantryItemRouter::list_pantry_items).post(PantryItemRouter::create_pantry_item),
            )
            .route(
                "/:id",
                get(PantryItemRouter::get_pantry_item)
                    .put(PantryItemRouter::update_pantry_item)
                    .delete(PantryItemRouter::delete_pantry_item),
            )
    }

    async fn create_pantry_item(
        State(state): State<AppState>,
        Json(payload): Json<CreatePayload>,
    ) -> (StatusCode, Json<Option<PantryItemResponse>>) {
        match state.db_client.create_pantry_item(payload.into()).await {
            Ok(pantry_item) => {
                log::info!("PantryItem with id {:?} created", pantry_item.id.to_string());
                (StatusCode::CREATED, Json(Some(pantry_item.into())))
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

    async fn list_pantry_items(
        State(state): State<AppState>,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<PantryItemListResponse>>) {
        match state.db_client.list_pantry_items(query_params.into()).await {
            Ok(pantry_items) => {
                log::info!("{:?} pantry_items collected", pantry_items.items.len());
                (StatusCode::OK, Json(Some(pantry_items.into())))
            }
            Err(err) => {
                log::error!("{}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
            }
        }
    }

    async fn get_pantry_item(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
    ) -> (StatusCode, Json<Option<PantryItemResponse>>) {
        match state.db_client.get_pantry_item(id).await {
            Ok(pantry_item) => {
                log::info!("Got pantry_item with id {:?}", pantry_item.id);
                (StatusCode::OK, Json(Some(pantry_item.into())))
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

    async fn update_pantry_item(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
        Json(payload): Json<UpdatePayload>,
    ) -> (StatusCode, Json<Option<PantryItemResponse>>) {
        match state.db_client.update_pantry_item(id, payload.into()).await {
            Ok(pantry_item) => {
                log::info!("Updated pantry_item with id {id:?}");
                (StatusCode::OK, Json(Some(pantry_item.into())))
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

    async fn delete_pantry_item(State(state): State<AppState>, Path(id): Path<Uuid>) -> StatusCode {
        match state.db_client.delete_pantry_item(id).await {
            Ok(()) => {
                log::info!("Deleted pantry_item with id {:?}", id);
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
