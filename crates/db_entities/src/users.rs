//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub password_hash: String,
    pub admin: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::pantry_items::Entity")]
    PantryItems,
    #[sea_orm(has_many = "super::recipes::Entity")]
    RecipeUsers,
}

impl Related<super::pantry_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PantryItems.def()
    }
}

impl Related<super::recipes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RecipeUsers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
