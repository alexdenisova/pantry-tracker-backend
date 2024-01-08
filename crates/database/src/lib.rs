use sea_orm::DatabaseConnection;

pub mod ingredients;
pub mod pantry_items;
pub mod recipe_ingredients;
pub mod recipe_users;
pub mod recipes;
pub mod users;

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

pub trait DatabaseCRUD:
    ingredients::DatabaseCRUD
    + pantry_items::DatabaseCRUD
    + recipe_ingredients::DatabaseCRUD
    + recipe_users::DatabaseCRUD
    + recipes::DatabaseCRUD
    + users::DatabaseCRUD
{
}

impl DatabaseCRUD for DBClient {}
