pub mod dto;

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use self::dto::{CreateDto, IngredientDto, IngredientsListDto, ListParamsDto};
use crate::database::{
    errors::{CreateError, DeleteError, GetError, ListError},
    DBClient,
};
use db_entities::ingredients::{ActiveModel, Column, Entity, Model};
use migrations::{Expr, Func};

#[async_trait]
pub trait DatabaseCRUD {
    async fn create_ingredient(&self, request: CreateDto) -> Result<IngredientDto, CreateError>;
    async fn get_ingredient(&self, id: Uuid) -> Result<IngredientDto, GetError>;
    async fn list_ingredients(
        &self,
        list_params: ListParamsDto,
    ) -> Result<IngredientsListDto, ListError>;
    async fn delete_ingredient(&self, id: Uuid) -> Result<(), DeleteError>;
}

#[async_trait]
impl DatabaseCRUD for DBClient {
    async fn create_ingredient(&self, request: CreateDto) -> Result<IngredientDto, CreateError> {
        let model: Model = request.into();
        let id = model.id;
        let active_model: ActiveModel = model.into();
        Ok(active_model
            .insert(&self.database_connection)
            .await
            .map_err(|err| {
                if let DbErr::RecordNotInserted = err {
                    CreateError::AlreadyExist { id }
                } else {
                    CreateError::Unexpected { error: err.into() }
                }
            })?
            .into())
    }
    async fn get_ingredient(&self, id: Uuid) -> Result<IngredientDto, GetError> {
        Ok(Entity::find_by_id(id)
            .one(&self.database_connection)
            .await
            .map_err(|err| GetError::Unexpected {
                id,
                error: err.into(),
            })?
            .ok_or(GetError::NotFound { id })?
            .into())
    }
    async fn list_ingredients(
        &self,
        list_params: ListParamsDto,
    ) -> Result<IngredientsListDto, ListError> {
        Ok(IngredientsListDto {
            items: match list_params.name {
                Some(value) => Entity::find().filter(
                    Expr::expr(Func::lower(Expr::col(Column::Name)))
                        .eq(value.to_lowercase().to_string()),
                ),
                None => Entity::find(),
            }
            .order_by_desc(Column::Name)
            .order_by_desc(Column::CreatedAt)
            .all(&self.database_connection)
            .await
            .map_err(|err| ListError::Unexpected { error: err.into() })?
            .into_iter()
            .map(Into::into)
            .collect(),
        })
    }
    async fn delete_ingredient(&self, id: Uuid) -> Result<(), DeleteError> {
        if Entity::delete_by_id(id)
            .exec(&self.database_connection)
            .await
            .map_err(|err| DeleteError::Unexpected {
                id,
                error: err.into(),
            })?
            .rows_affected
            == 0
        {
            Err(DeleteError::NotFound { id })
        } else {
            Ok(())
        }
    }
}
