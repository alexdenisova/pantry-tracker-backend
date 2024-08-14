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
use crate::server::routes::utils::verify_password;
use crate::server::routes::COOKIE_KEY;
use crate::server::state::AppState;
use payload::LoginPayload;

const SESSION_TTL_DAYS: u16 = 7;
pub struct LoginRouter {}

impl LoginRouter {
    pub fn get() -> Router<AppState> {
        Router::new().route("/", post(LoginRouter::login).delete(LoginRouter::logout))
    }

    async fn login(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<LoginPayload>,
    ) -> Result<(CookieJar, Redirect), StatusCode> {
        let username = payload.username.clone();
        let password = payload.password.clone().unwrap_or_default();
        match state.db_client.list_users(payload.into()).await {
            Ok(users) => {
                if users.items.is_empty() {
                    log::info!("User {:?} does not exist", username);
                    return Err(StatusCode::NOT_FOUND);
                }
                let user = &users.items[0];
                if verify_password(&password, &user.password_hash) {
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
                } else {
                    log::info!("Wrong password from {:?}", username);
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
            Err(err) => {
                log::error!("{}", err.to_string());
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    async fn logout(
        State(state): State<AppState>,
        jar: CookieJar,
    ) -> Result<(CookieJar, Redirect), StatusCode> {
        if let Some(session_id) = jar.get(COOKIE_KEY) {
            let session_id = session_id.value_trimmed();
            if let Ok(true) = state.session_is_valid(session_id).await {
                if delete_session(session_id, &state.redis_sender)
                    .await
                    .is_err()
                {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                return Ok((jar.remove(COOKIE_KEY), Redirect::to("/login")));
            }
        }
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn create_session(user_id: Uuid, redis_sender: &Sender<RedisCommand>) -> AnyResult<String> {
    let session_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    redis_sender.set(&session_id, &user_id.to_string(), Some(SESSION_TTL_DAYS)).await?;
    log::info!("Session created");
    Ok(session_id)
}

async fn delete_session(session_id: &str, redis_sender: &Sender<RedisCommand>) -> AnyResult<()> {
    redis_sender.delete(session_id).await
}
