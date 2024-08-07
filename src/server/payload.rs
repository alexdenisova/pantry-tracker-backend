use serde::{Deserialize, Serialize};

use crate::database::dto::MetadataDto;

pub const DEFAULT_PER_PAGE: u64 = 25;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct MetadataResponse {
    pub page: u64,
    pub per_page: u64,
    pub page_count: u64,
    pub total_count: u64,
}

impl From<MetadataDto> for MetadataResponse {
    fn from(val: MetadataDto) -> Self {
        MetadataResponse {
            page: val.page,
            per_page: val.per_page,
            page_count: val.page_count,
            total_count: val.total_count,
        }
    }
}
