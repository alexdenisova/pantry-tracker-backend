use std::sync::Arc;

use super::routes::DaoTrait;

#[derive(Clone)]
pub struct AppState {
    pub dao: Arc<dyn DaoTrait + Send + Sync>,
}

impl AppState {
    pub fn new(database: impl DaoTrait + Send + Sync + 'static) -> Self {
        Self {
            dao: Arc::new(database),
        }
    }
}
