pub mod routes;
mod state;

use axum::routing::get;
use axum::{extract::State, http::StatusCode, serve, Router};
use http::{header::CONTENT_TYPE, Method};
pub use state::AppState;
use thiserror::Error;
use tokio::net::{TcpListener, ToSocketAddrs};
use tower_http::cors::{Any, CorsLayer};

use self::routes::ingredients::IngredientRouter;
use self::routes::pantry_items::PantryItemRouter;
use self::routes::parse_ingredients::ParseIngredientsRouter;
use self::routes::possible_recipes::PossibleRecipesRouter;
use self::routes::recipe_ingredients::RecipeIngredientRouter;
use self::routes::recipe_users::RecipeUserRouter;
use self::routes::recipes::RecipeRouter;
use self::routes::users::UserRouter;

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

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_origin(Any)
            .allow_headers([CONTENT_TYPE]);

        let router: Router = Router::new()
            .route("/health", get(health))
            .nest("/ingredients", IngredientRouter::get())
            .nest("/pantry_items", PantryItemRouter::get())
            .nest("/parse_ingredients", ParseIngredientsRouter::get())
            .nest("/possible_recipes", PossibleRecipesRouter::list())
            .nest("/recipes", RecipeRouter::get())
            .nest("/recipe_ingredients", RecipeIngredientRouter::get())
            .nest("/recipe_users", RecipeUserRouter::get())
            .nest("/users", UserRouter::get())
            .with_state(self.state)
            .fallback(Server::fallback)
            .layer(cors);

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
        log::debug!("Healthy");
        StatusCode::OK
    } else {
        log::warn!("Unhealthy");
        StatusCode::SERVICE_UNAVAILABLE
    }
}
