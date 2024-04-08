use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use payload::{
    CreatePayload, RecipeIngredientListResponse, RecipeIngredientResponse, ListQueryParams, UpdatePayload,
};

use crate::database::errors::{CreateError, DeleteError, GetError, UpdateError};
use crate::server::state::AppState;
use uuid::Uuid;

mod payload;

pub struct RecipeIngredientRouter {}

impl RecipeIngredientRouter {
    pub fn get() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(RecipeIngredientRouter::list_recipe_ingredients).post(RecipeIngredientRouter::create_recipe_ingredient),
            )
            .route(
                "/:id",
                get(RecipeIngredientRouter::get_recipe_ingredient)
                    .put(RecipeIngredientRouter::update_recipe_ingredient)
                    .delete(RecipeIngredientRouter::delete_recipe_ingredient),
            )
    }

    async fn create_recipe_ingredient(
        State(state): State<AppState>,
        Json(payload): Json<CreatePayload>,
    ) -> (StatusCode, Json<Option<RecipeIngredientResponse>>) {
        match state.db_client.create_recipe_ingredient(payload.into()).await {
            Ok(recipe_ingredient) => {
                log::info!("Recipe ingredient with id {:?} created", recipe_ingredient.id.to_string());
                (StatusCode::CREATED, Json(Some(recipe_ingredient.into())))
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

    pub async fn list_recipe_ingredients(
        State(state): State<AppState>,
        Query(query_params): Query<ListQueryParams>,
    ) -> (StatusCode, Json<Option<RecipeIngredientListResponse>>) {
        match state.db_client.list_recipe_ingredients(query_params.into()).await {
            Ok(recipe_ingredients) => {
                log::info!("{:?} recipe ingredients collected", recipe_ingredients.items.len());
                (StatusCode::OK, Json(Some(recipe_ingredients.into())))
            }
            Err(err) => {
                log::error!("{}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
            }
        }
    }

    async fn get_recipe_ingredient(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
    ) -> (StatusCode, Json<Option<RecipeIngredientResponse>>) {
        match state.db_client.get_recipe_ingredient(id).await {
            Ok(recipe_ingredient) => {
                log::info!("Got recipe ingredient with id {:?}", recipe_ingredient.id);
                (StatusCode::OK, Json(Some(recipe_ingredient.into())))
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

    async fn update_recipe_ingredient(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
        Json(payload): Json<UpdatePayload>,
    ) -> (StatusCode, Json<Option<RecipeIngredientResponse>>) {
        match state.db_client.update_recipe_ingredient(id, payload.into()).await {
            Ok(recipe_ingredient) => {
                log::info!("Updated recipe ingredient with id {id:?}");
                (StatusCode::OK, Json(Some(recipe_ingredient.into())))
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

    async fn delete_recipe_ingredient(State(state): State<AppState>, Path(id): Path<Uuid>) -> StatusCode {
        match state.db_client.delete_recipe_ingredient(id).await {
            Ok(()) => {
                log::info!("Deleted recipe ingredient with id {:?}", id);
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
