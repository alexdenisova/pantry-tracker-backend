pub mod errors;
pub mod ingredients;
pub mod pantry_items;
pub mod recipe_ingredients;
pub mod recipes;
pub mod users;

use crate::database::errors::HealthcheckError;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;

pub struct DBClient {
    database_connection: DatabaseConnection,
}

impl DBClient {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        DBClient {
            database_connection: db_connection,
        }
    }
}

#[async_trait]
pub trait DBHealth {
    async fn health(&self) -> Result<(), HealthcheckError>;
}

#[async_trait]
impl DBHealth for DBClient {
    async fn health(&self) -> Result<(), HealthcheckError> {
        self.database_connection
            .ping()
            .await
            .map_err(|err| HealthcheckError::Unexpected { error: err.into() })
    }
}

pub trait DBTrait:
    DBHealth
    + ingredients::DatabaseCRUD
    + pantry_items::DatabaseCRUD
    + recipe_ingredients::DatabaseCRUD
    + recipes::DatabaseCRUD
    + users::DatabaseCRUD
{
}

impl DBTrait for DBClient {}
