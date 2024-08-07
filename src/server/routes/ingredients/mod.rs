mod payload;

use axum::extract::Query;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use uuid::Uuid;

use crate::server::routes::errors::AppError;
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use payload::{CreatePayload, IngredientListResponse, IngredientResponse, ListQueryParams};

pub struct IngredientRouter {}

impl IngredientRouter {
    pub fn router() -> Router<AppState> {
        Router::new()
            .route(
                "/",
                get(IngredientRouter::list).post(IngredientRouter::create),
            )
            .route(
                "/:id",
                get(IngredientRouter::get).delete(IngredientRouter::delete),
            )
    }

    async fn create(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<CreatePayload>,
    ) -> Result<(StatusCode, Json<IngredientResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if state.session_is_valid(session_id.value_trimmed()).await? {
                let ingredient = state.db_client.create_ingredient(payload.into()).await?;
                log::info!("Ingredient with id {:?} created", ingredient.id.to_string());
                return Ok((StatusCode::CREATED, Json(ingredient.into())));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn list(
        State(state): State<AppState>,
        jar: CookieJar,
        Query(query_params): Query<ListQueryParams>,
    ) -> Result<(StatusCode, Json<IngredientListResponse>), AppError> {
        if query_params.name.is_some() && query_params.name_contains.is_some() {
            return Err(AppError::UnprocessableEntity {
                error: eyre!("Only one of name or name_contains can be defined."),
            });
        }
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if state.session_is_valid(session_id.value_trimmed()).await? {
                let list_params = query_params.into();
                let ingredients: Vec<IngredientResponse> =
                    state.db_client.list_ingredients(&list_params).await?.into();
                log::info!("{:?} ingredients collected", ingredients.len());
                let metadata = state
                    .db_client
                    .get_ingredients_metadata(&list_params)
                    .await?
                    .into();
                return Ok((
                    StatusCode::OK,
                    Json(IngredientListResponse::from(ingredients, metadata)),
                ));
            }
        }
        Err(AppError::Unauthorized)
    }

    async fn get(
        State(state): State<AppState>,
        jar: CookieJar,
        Path(id): Path<Uuid>,
    ) -> Result<(StatusCode, Json<IngredientResponse>), AppError> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            if state.session_is_valid(session_id.value_trimmed()).await? {
                let ingredient = state.db_client.get_ingredient(id).await?;
                log::info!("Got ingredient with id {:?}", ingredient.id);
                return Ok((StatusCode::OK, Json(ingredient.into())));
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
                    state.db_client.delete_ingredient(id).await?;
                    log::info!("Deleted ingredient with id {:?}", id);
                    return Ok(StatusCode::NO_CONTENT);
                }
            }
        }
        Err(AppError::Unauthorized)
    }
}
