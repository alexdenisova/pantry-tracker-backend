use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct MetadataDto {
    pub page: u64,
    pub per_page: u64,
    pub page_count: u64,
    pub total_count: u64,
}
