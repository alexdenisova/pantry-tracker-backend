mod payload;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::database::errors::{CreateError, DeleteError, GetError, UpdateError};
use crate::server::routes::utils::VerifyError;
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
    ) -> (StatusCode, Json<Option<PantryItemResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if !validate_quantity(
                    payload.quantity,
                    payload.weight_grams,
                    payload.volume_milli_litres,
                ) {
                    log::debug!("Amount is not valid");
                    return (StatusCode::UNPROCESSABLE_ENTITY, Json(None));
                }
                match state
                    .db_client
                    .create_pantry_item(payload.into_dto(user_id))
                    .await
                {
                    Ok(pantry_item) => {
                        log::info!(
                            "Pantry item with id {:?} created",
                            pantry_item.id.to_string()
                        );
                        return (StatusCode::CREATED, Json(Some(pantry_item.into())));
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
        log::debug!("Could not create pantry item: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn list_pantry_items(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<PantryItemListResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                let user_id = if query_params.all.is_some_and(|x| x) {
                    if !state.user_is_admin(user_id).await.is_ok_and(|x| x) {
                        return (StatusCode::UNAUTHORIZED, Json(None));
                    }
                    None
                } else {
                    Some(user_id)
                };
                let name_pattern = query_params.name_contains.clone();
                match state
                    .db_client
                    .list_pantry_items(query_params.into_dto(user_id))
                    .await
                {
                    Ok(pantry_items) => {
                        if let Some(pattern) = name_pattern {
                            let mut filtered_items = Vec::new();
                            for item in pantry_items.items {
                                let ingredient_name = state
                                    .db_client
                                    .get_ingredient(item.ingredient_id)
                                    .await
                                    .unwrap()
                                    .name;
                                if ingredient_name.to_lowercase().contains(&pattern) {
                                    filtered_items.push(item.into());
                                }
                            }
                            log::info!("{:?} pantry items collected", filtered_items.len());
                            return (
                                StatusCode::OK,
                                Json(Some(PantryItemListResponse {
                                    items: filtered_items,
                                })),
                            );
                        }
                        log::info!("{:?} pantry items collected", pantry_items.items.len());
                        return (StatusCode::OK, Json(Some(pantry_items.into())));
                    }
                    Err(err) => {
                        log::error!("{}", err.to_string());
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
                    }
                }
            }
        }
        log::debug!("Could not list pantry items: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn get_pantry_item(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> (StatusCode, Json<Option<PantryItemResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                match state.db_client.get_pantry_item(id).await {
                    Ok(pantry_item) => {
                        if pantry_item.user_id == user_id {
                            log::info!("Got pantry item with id {:?}", pantry_item.id);
                            return (StatusCode::OK, Json(Some(pantry_item.into())));
                        }
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
        log::debug!("Could not get pantry item: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn update_pantry_item(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
        Json(payload): Json<UpdatePayload>,
    ) -> (StatusCode, Json<Option<PantryItemResponse>>) {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if let Err(err) = verified_user(&state, id, user_id).await {
                    return (err.into(), Json(None));
                }
                if !validate_quantity(
                    payload.quantity,
                    payload.weight_grams,
                    payload.volume_milli_litres,
                ) {
                    return (StatusCode::UNPROCESSABLE_ENTITY, Json(None));
                }
                match state
                    .db_client
                    .update_pantry_item(id, payload.into_dto(user_id))
                    .await
                {
                    Ok(pantry_item) => {
                        log::info!("Updated pantry item with id {id:?}");
                        return (StatusCode::OK, Json(Some(pantry_item.into())));
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
        log::debug!("Could not update pantry item: user unauthorized");
        (StatusCode::UNAUTHORIZED, Json(None))
    }

    async fn delete_pantry_item(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> StatusCode {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if let Ok(Some(user_id)) = state.get_sessions_user(session_id.value_trimmed()).await {
                if let Err(err) = verified_user(&state, id, user_id).await {
                    return err.into();
                }
                match state.db_client.delete_pantry_item(id).await {
                    Ok(()) => {
                        log::info!("Deleted pantry item with id {:?}", id);
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
        log::debug!("Could not delete pantry item: user unauthorized");
        StatusCode::UNAUTHORIZED
    }
}

async fn verified_user(state: &AppState, id: Uuid, user_id: Uuid) -> Result<(), VerifyError> {
    if let Ok(true) = state.user_is_admin(user_id).await {
        return Ok(());
    }
    match state.db_client.get_pantry_item(id).await {
        Ok(pantry_item) => {
            if pantry_item.user_id == user_id {
                log::info!("Got pantry item with id {:?}", pantry_item.id);
                return Ok(());
            }
            Err(VerifyError::Unauthorized)
        }
        Err(err) => {
            if let GetError::NotFound { .. } = err {
                log::error!("{}", err.to_string());
                return Err(VerifyError::NotFound);
            }
            log::error!("{}", err.to_string());
            Err(VerifyError::InternalServerError)
        }
    }
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
