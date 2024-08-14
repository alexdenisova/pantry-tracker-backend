pub mod routes;
mod state;

use axum::routing::get;
use axum::{extract::State, http::StatusCode, serve, Router};
pub use state::AppState;
use thiserror::Error;
use tokio::net::{TcpListener, ToSocketAddrs};

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

        let router: Router = Router::new()
            .route("/health", get(health))
            .nest("/users", UserRouter::get())
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
    if state.dao.health().await.is_ok() {
        log::debug!("Healthy");
        StatusCode::OK
    } else {
        log::warn!("Unhealthy");
        StatusCode::SERVICE_UNAVAILABLE
    }
}
