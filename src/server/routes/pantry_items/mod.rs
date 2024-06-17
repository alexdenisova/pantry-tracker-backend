mod payload;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use uuid::Uuid;

use crate::server::routes::errors::{AppError, VerifyError};
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use payload::{
    CreatePayload, ListQueryParams, PantryItemListResponse, PantryItemResponse, UpdatePayload,
};

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
        jar: CookieJar,
        Json(payload): Json<CreatePayload>,
    ) -> Result<(StatusCode, Json<PantryItemResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if !validate_quantity(
                    payload.quantity,
                    payload.weight_grams,
                    payload.volume_milli_litres,
                ) {
                    return Err(AppError::UnprocessableEntity { error: eyre!("Invalid amount. Must indicate only one of quantity, weight_grams or volume_milli_litres.") });
                }
                let pantry_item = state
                    .db_client
                    .create_pantry_item(payload.into_dto(user_id))
                    .await
                    .map_err(Into::<AppError>::into)?;
                log::info!(
                    "Pantry item with id {:?} created",
                    pantry_item.id.to_string()
                );
                return Ok((StatusCode::CREATED, Json(pantry_item.into())));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn list_pantry_items(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> Result<(StatusCode, Json<PantryItemListResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let name_pattern = query_params.name_contains.clone();
                let mut pantry_items = state
                    .db_client
                    .list_pantry_items(query_params.into_dto(user_id))
                    .await
                    .map_err(Into::<AppError>::into)?;
                if let Some(pattern) = name_pattern {
                    let mut filtered_items = Vec::new();
                    for item in pantry_items.items {
                        let ingredient_name = state
                            .db_client
                            .get_ingredient(item.ingredient_id)
                            .await
                            .map_err(Into::<AppError>::into)?
                            .name;
                        if ingredient_name.to_lowercase().contains(&pattern) {
                            filtered_items.push(item);
                        }
                    }
                    pantry_items.items = filtered_items;
                }
                log::info!("{:?} pantry items collected", pantry_items.items.len());
                return Ok((StatusCode::OK, Json(pantry_items.into())));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn get_pantry_item(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> Result<(StatusCode, Json<PantryItemResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let pantry_item = state
                    .db_client
                    .get_pantry_item(id)
                    .await
                    .map_err(Into::<AppError>::into)?;
                if pantry_item.user_id == user_id {
                    log::info!("Got pantry item with id {:?}", pantry_item.id);
                    return Ok((StatusCode::OK, Json(pantry_item.into())));
                }
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn update_pantry_item(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
        Json(payload): Json<UpdatePayload>,
    ) -> Result<(StatusCode, Json<PantryItemResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                verified_user(&state, id, user_id)
                    .await
                    .map_err(Into::<AppError>::into)?;
                if !validate_quantity(
                    payload.quantity,
                    payload.weight_grams,
                    payload.volume_milli_litres,
                ) {
                    return Err(AppError::UnprocessableEntity { error: eyre!("Invalid amount. Must indicate only one of quantity, weight_grams or volume_milli_litres.") });
                }
                let pantry_item = state
                    .db_client
                    .update_pantry_item(id, payload.into_dto(user_id))
                    .await
                    .map_err(Into::<AppError>::into)?;
                log::info!("Updated pantry item with id {id:?}");
                return Ok((StatusCode::OK, Json(pantry_item.into())));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn delete_pantry_item(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> Result<StatusCode, AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                verified_user(&state, id, user_id)
                    .await
                    .map_err(Into::<AppError>::into)?;
                state
                    .db_client
                    .delete_pantry_item(id)
                    .await
                    .map_err(Into::<AppError>::into)?;
                {
                    log::info!("Deleted pantry item with id {:?}", id);
                    return Ok(StatusCode::NO_CONTENT);
                }
            }
        }
        Err(AppError::Unauthorized)
    }
}

async fn verified_user(state: &AppState, id: Uuid, user_id: Uuid) -> Result<(), VerifyError> {
    if let Ok(true) = state.user_is_admin(user_id).await {
        return Ok(());
    }
    let pantry_item = state
        .db_client
        .get_pantry_item(id)
        .await
        .map_err(Into::<VerifyError>::into)?;
    if pantry_item.user_id == user_id {
        log::info!("Got pantry item with id {:?}", pantry_item.id);
        return Ok(());
    }
    Err(VerifyError::Unauthorized)
}

fn validate_quantity(
    quantity: Option<i32>,
    weight_grams: Option<i32>,
    volume_milli_litres: Option<i32>,
) -> bool {
    quantity.is_none() && weight_grams.is_none() && volume_milli_litres.is_none()
        || quantity.is_some() && weight_grams.is_none() && volume_milli_litres.is_none()
        || quantity.is_none() && weight_grams.is_some() && volume_milli_litres.is_none()
        || quantity.is_none() && weight_grams.is_none() && volume_milli_litres.is_some()
}
