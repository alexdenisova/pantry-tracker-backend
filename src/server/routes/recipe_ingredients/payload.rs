use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::recipe_ingredients::dto::{
    CreateDto, ListParamsDto, RecipeIngredientDto, RecipeIngredientsListDto, UpdateDto,
};
use crate::server::payload::{MetadataResponse, DEFAULT_PER_PAGE};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePayload {
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
    pub amount: Option<String>,
    pub unit: Option<String>,
    pub optional: bool,
}

impl From<CreatePayload> for CreateDto {
    fn from(val: CreatePayload) -> Self {
        CreateDto {
            recipe_id: val.recipe_id,
            ingredient_id: val.ingredient_id,
            amount: val.amount,
            unit: val.unit,
            optional: val.optional,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdatePayload {
    pub ingredient_id: Uuid,
    pub amount: Option<String>,
    pub unit: Option<String>,
    pub optional: bool,
}

impl From<UpdatePayload> for UpdateDto {
    fn from(val: UpdatePayload) -> Self {
        UpdateDto {
            ingredient_id: val.ingredient_id,
            amount: val.amount,
            unit: val.unit,
            optional: val.optional,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ListQueryParams {
    pub recipe_id: Option<Uuid>,
    pub ingredient_id: Option<Uuid>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl From<ListQueryParams> for ListParamsDto {
    fn from(val: ListQueryParams) -> Self {
        ListParamsDto {
            recipe_id: val.recipe_id,
            limit: val.per_page.unwrap_or(DEFAULT_PER_PAGE),
            offset: val.per_page.unwrap_or(DEFAULT_PER_PAGE) * (val.page.unwrap_or(1) - 1),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RecipeIngredientResponse {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
    // pub ingredient_name: String,
    pub amount: Option<String>,
    pub unit: Option<String>,
    pub optional: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<RecipeIngredientDto> for RecipeIngredientResponse {
    fn from(val: RecipeIngredientDto) -> Self {
        RecipeIngredientResponse {
            id: val.id,
            recipe_id: val.recipe_id,
            ingredient_id: val.ingredient_id,
            amount: val.amount,
            unit: val.unit,
            optional: val.optional,
            created_at: val.created_at,
            updated_at: val.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RecipeIngredientListResponse {
    #[serde(rename = "_metadata")]
    pub metadata: MetadataResponse,
    pub items: Vec<RecipeIngredientResponse>,
}

impl From<RecipeIngredientsListDto> for Vec<RecipeIngredientResponse> {
    fn from(val: RecipeIngredientsListDto) -> Self {
        val.items.into_iter().map(Into::into).collect()
    }
}

impl RecipeIngredientListResponse {
    pub fn from(items: Vec<RecipeIngredientResponse>, metadata: MetadataResponse) -> Self {
        RecipeIngredientListResponse { metadata, items }
    }
}
