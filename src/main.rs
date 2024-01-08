use chrono::Utc;
use database::{DBClient, DatabaseCRUD};
use futures::executor::block_on;
use migrations::{Migrator, MigratorTrait};
use sea_orm::*;
use sea_orm_migration::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

const DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5432";
const DB_NAME: &str = "food_db";

pub struct State {
    pub repository: Arc<dyn DatabaseCRUD + Send + Sync>,
}

async fn run() -> Result<(), DbErr> {
    let db = Database::connect(DATABASE_URL).await?;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("DROP DATABASE IF EXISTS \"{}\";", DB_NAME),
    ))
    .await?;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("CREATE DATABASE \"{}\";", DB_NAME),
    ))
    .await?;

    let url = format!("{}/{}", DATABASE_URL, DB_NAME);
    let db = &Database::connect(&url).await?;

    let schema_manager = SchemaManager::new(db); // To investigate the schema
    Migrator::refresh(db).await?;

    let database_client = DBClient::new(Database::connect(&url).await?);
    let state = State {
        repository: Arc::new(database_client),
    };
    state
        .repository
        .create_user(database::users::Request {
            id: Uuid::new_v4(),
            name: "me".to_owned(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        })
        .await?;
    Ok(())
}

fn main() {
    if let Err(err) = block_on(run()) {
        panic!("{}", err);
    }
}
