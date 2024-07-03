use color_eyre::Result as AnyResult;
use sea_orm::DatabaseConnection;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::database::errors::GetError;
use crate::database::{DBClient, DBTrait};
use crate::redis::{RedisCommand, RedisCommands, RedisResult};

#[derive(Clone)]
pub struct AppState {
    pub db_client: Arc<dyn DBTrait + Send + Sync>,
    pub redis_sender: Sender<RedisCommand>,
}

impl AppState {
    pub fn new(db_connection: DatabaseConnection, redis_sender: Sender<RedisCommand>) -> Self {
        let db_client = DBClient::new(db_connection);
        Self {
            db_client: Arc::new(db_client),
            redis_sender,
        }
    }
    /// Returns the `user_id`
    pub async fn session_is_valid(&self, session_id: &str) -> RedisResult<bool> {
        Ok(self.redis_sender.get(session_id).await?.is_some())
    }
    /// Returns the `user_id`
    pub async fn get_sessions_user(&self, session_id: &str) -> AnyResult<Option<Uuid>> {
        match self.redis_sender.get(session_id).await? {
            Some(id) => Ok(Some(Uuid::from_str(&id)?)),
            None => Ok(None),
        }
    }

    pub async fn user_is_admin(&self, user_id: Uuid) -> Result<bool, GetError> {
        Ok(self.db_client.get_user(user_id).await?.admin)
    }
}
