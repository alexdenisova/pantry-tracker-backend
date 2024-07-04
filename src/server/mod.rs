pub mod routes;
mod state;

use axum::routing::get;
use axum::{extract::State, http::StatusCode, serve, Router};
use thiserror::Error;
use tokio::net::{TcpListener, ToSocketAddrs};

use crate::server::routes::login::LoginRouter;

use self::routes::ingredients::IngredientRouter;
use self::routes::pantry_items::PantryItemRouter;
use self::routes::parse_ingredients::ParseIngredientsRouter;
use self::routes::parse_recipe_link::ParsedRecipeLinkRouter;
use self::routes::recipe_ingredients::RecipeIngredientRouter;
use self::routes::recipes::RecipeRouter;
use self::routes::users::UserRouter;
pub use state::AppState;

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ServerError {
    #[error("Problem with listener binding: {message:?}")]
    ListenerError { message: String },
    #[error("Problem with server start: {message:?}")]
    RouterServeError { message: String },
}

pub struct Server {
    state: AppState,
}

impl Server {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn run(self, bind_address: impl ToSocketAddrs) -> ServerResult<()> {
        let listener = TcpListener::bind(bind_address).await.map_err(|err| {
            log::error!("{}", err.to_string());
            ServerError::ListenerError {
                message: err.to_string(),
            }
        })?;

        let router: Router = Router::new()
            .route("/health", get(health))
            .nest("/login", LoginRouter::router())
            .nest("/ingredients", IngredientRouter::router())
            .nest("/pantry_items", PantryItemRouter::router())
            .nest("/parse_ingredients", ParseIngredientsRouter::router())
            .nest("/parse_recipe_link", ParsedRecipeLinkRouter::router())
            .nest("/recipes", RecipeRouter::router())
            .nest("/recipe_ingredients", RecipeIngredientRouter::router())
            .nest("/users", UserRouter::router())
            .with_state(self.state)
            .fallback(Server::fallback);

        serve(listener, router).await.map_err(|err| {
            log::error!("{}", err.to_string());
            ServerError::RouterServeError {
                message: err.to_string(),
            }
        })?;

        Ok(())
    }

    #[allow(clippy::unused_async)]
    async fn fallback() -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

async fn health(State(state): State<AppState>) -> StatusCode {
    if state.db_client.health().await.is_ok() {
        StatusCode::OK
    } else {
        log::warn!("Unhealthy");
        StatusCode::SERVICE_UNAVAILABLE
    }
}
