use async_trait::async_trait;
use database::{errors::HealthcheckError, DBClient, DBTrait};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::server::routes::DatabaseHealth;

pub struct DatabaseClient {
    pub client: Arc<dyn DBTrait + Send + Sync>,
}

impl DatabaseClient {
    pub fn new(connection: DatabaseConnection) -> Self {
        let client = DBClient::new(connection);
        Self {
            client: Arc::new(client),
        }
    }
}

#[async_trait]
impl DatabaseHealth for DatabaseClient {
    async fn health(&self) -> Result<(), HealthcheckError> {
        self.client.health().await
    }
}

impl crate::server::routes::DaoTrait for DatabaseClient {}
