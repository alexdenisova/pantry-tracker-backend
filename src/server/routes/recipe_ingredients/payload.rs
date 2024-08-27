use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::recipe_ingredients::dto::{
    CreateDto, ListParamsDto, RecipeIngredientDto, RecipeIngredientJoinDto,
    RecipeIngredientsListDto, UpdateDto,
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
    pub name_contains: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl ListQueryParams {
    pub fn into_dto(self, user_id: Option<Uuid>) -> ListParamsDto {
        ListParamsDto {
            recipe_id: self.recipe_id,
            user_id,
            name_contains: self.name_contains,
            limit: self.per_page.unwrap_or(DEFAULT_PER_PAGE),
            offset: self.per_page.unwrap_or(DEFAULT_PER_PAGE) * (self.page.unwrap_or(1) - 1),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RecipeIngredientResponse {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
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
pub struct RecipeIngredientJoinResponse {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub recipe_name: String,
    pub ingredient_id: Uuid,
    pub ingredient_name: String,
    pub amount: Option<String>,
    pub unit: Option<String>,
    pub optional: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<RecipeIngredientJoinDto> for RecipeIngredientJoinResponse {
    fn from(val: RecipeIngredientJoinDto) -> Self {
        RecipeIngredientJoinResponse {
            id: val.id,
            recipe_id: val.recipe_id,
            recipe_name: val.recipe_name,
            ingredient_id: val.ingredient_id,
            ingredient_name: val.ingredient_name,
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
    pub items: Vec<RecipeIngredientJoinResponse>,
}

impl From<RecipeIngredientsListDto> for Vec<RecipeIngredientJoinResponse> {
    fn from(val: RecipeIngredientsListDto) -> Self {
        val.items.into_iter().map(Into::into).collect()
    }
}

impl RecipeIngredientListResponse {
    pub fn from(items: Vec<RecipeIngredientJoinResponse>, metadata: MetadataResponse) -> Self {
        RecipeIngredientListResponse { metadata, items }
    }
}
