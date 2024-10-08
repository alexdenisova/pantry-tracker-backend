//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // TODO: 
    // #[sea_orm(has_many = "super::pantry_item_categories::Entity")]
    // PantryItemCategories,
    #[sea_orm(has_many = "super::recipe_categories::Entity")]
    RecipeCategories,
}

// impl Related<super::pantry_item_categories::Entity> for Entity {
//     fn to() -> RelationDef {
//         Relation::PantryItemCategories.def()
//     }
// }

impl Related<super::recipe_categories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RecipeCategories.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
