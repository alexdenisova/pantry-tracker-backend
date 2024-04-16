mod payload;

use axum::response::Redirect;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::post,
    Router,
};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use color_eyre::Result as AnyResult;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::redis::{RedisCommand, RedisCommands};
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use payload::LoginPayload;

pub struct LoginRouter {}

impl LoginRouter {
    pub fn get() -> Router<AppState> {
        Router::new().route("/", post(LoginRouter::login))
    }

    async fn login(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<LoginPayload>,
    ) -> Result<(CookieJar, Redirect), StatusCode> {
        let username = payload.username.clone();
        match state.db_client.list_users(payload.into()).await {
            Ok(users) => {
                if users.items.is_empty() {
                    log::info!("User {:?} does not exist", username);
                    return Err(StatusCode::NOT_FOUND);
                }
                let user = &users.items[0];
                log::info!("User {:?} logged in", username);
                match create_session(user.id, &state.redis_sender).await {
                    Ok(session_id) => Ok((
                        jar.add(Cookie::new(COOKIE_KEY, session_id)),
                        Redirect::to("/"),
                    )),
                    Err(err) => {
                        log::error!("{}", err.to_string());
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
            Err(err) => {
                log::error!("{}", err.to_string());
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

async fn create_session(user_id: Uuid, redis_sender: &Sender<RedisCommand>) -> AnyResult<String> {
    let session_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    redis_sender.set(&session_id, &user_id.to_string()).await?;
    log::info!("Session created");
    Ok(session_id)
}
