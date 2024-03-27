use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::database::{DBClient, DBTrait};

#[derive(Clone)]
pub struct AppState {
    pub db_client: Arc<dyn DBTrait + Send + Sync>,
}

impl AppState {
    pub fn new(connection: DatabaseConnection) -> Self {
        let client = DBClient::new(connection);
        Self {
            db_client: Arc::new(client),
        }
    }
}
