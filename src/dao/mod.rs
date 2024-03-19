pub mod db_client;
pub mod ingredients;
pub mod users;

use async_trait::async_trait;
use database::errors::HealthcheckError;

#[async_trait]
pub trait DatabaseHealth {
    async fn health(&self) -> Result<(), HealthcheckError>;
}

pub trait DaoTrait: DatabaseHealth + users::DaoTrait + ingredients::DaoTrait {}
