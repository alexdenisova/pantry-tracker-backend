//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "recipes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub prep_time_mins: Option<i32>,
    pub total_time_mins: Option<i32>,
    pub link: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub instructions: Option<String>,
    pub image: Option<String>, // TODO: make this link or file. Store files in S3
    pub last_cooked: Option<Date>,
    pub rating: Option<i32>,
    pub notes: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::recipe_ingredients::Entity")]
    RecipeIngredients,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::recipe_ingredients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RecipeIngredients.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
