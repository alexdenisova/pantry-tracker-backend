use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the FoodItems, Recipes tables.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Ingredients::Table)
                    .col(ColumnDef::new(Ingredients::Id).uuid().primary_key())
                    .col(ColumnDef::new(Ingredients::Name).string().not_null())
                    .col(
                        ColumnDef::new(Ingredients::CanBeEatenRaw)
                            .boolean()
                            .default(false)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Ingredients::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(ColumnDef::new(Users::Id).uuid().primary_key())
                    .col(ColumnDef::new(Users::Name).string().not_null())
                    .col(ColumnDef::new(Users::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Users::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(PantryItems::Table)
                    .col(ColumnDef::new(PantryItems::Id).uuid().primary_key())
                    .col(ColumnDef::new(PantryItems::IngredientId).uuid().not_null())
                    .col(ColumnDef::new(PantryItems::PurchaseDate).date())
                    .col(
                        ColumnDef::new(PantryItems::ExpirationDate)
                            .date()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PantryItems::Quantity).integer().not_null())
                    .col(ColumnDef::new(PantryItems::WeightGrams).integer())
                    .col(ColumnDef::new(PantryItems::VolumeMilliLitres).integer())
                    .col(ColumnDef::new(PantryItems::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(PantryItems::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PantryItems::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_tbl(PantryItems::Table)
                            .from_col(PantryItems::IngredientId)
                            .to_tbl(Ingredients::Table)
                            .to_col(Ingredients::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_tbl(PantryItems::Table)
                            .from_col(PantryItems::UserId)
                            .to_tbl(Users::Table)
                            .to_col(Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Recipes::Table)
                    .col(ColumnDef::new(Recipes::Id).uuid().primary_key())
                    .col(ColumnDef::new(Recipes::Name).string().not_null())
                    .col(ColumnDef::new(Recipes::CookingTimeMins).integer())
                    // .col(ColumnDef::new(Recipes::CookingTime).interval(None, None))
                    .col(ColumnDef::new(Recipes::Link).string())
                    .col(ColumnDef::new(Recipes::Instructions).text())
                    .col(ColumnDef::new(Recipes::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Recipes::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RecipeIngredients::Table)
                    .col(ColumnDef::new(RecipeIngredients::Id).uuid().primary_key())
                    .col(
                        ColumnDef::new(RecipeIngredients::RecipeId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RecipeIngredients::IngredientId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RecipeIngredients::Amount)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(RecipeIngredients::Unit).string().not_null())
                    .col(
                        ColumnDef::new(RecipeIngredients::Optional)
                            .boolean()
                            .default(false)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RecipeIngredients::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RecipeIngredients::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_tbl(RecipeIngredients::Table)
                            .from_col(RecipeIngredients::IngredientId)
                            .to_tbl(Ingredients::Table)
                            .to_col(Ingredients::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_tbl(RecipeIngredients::Table)
                            .from_col(RecipeIngredients::RecipeId)
                            .to_tbl(Recipes::Table)
                            .to_col(Recipes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RecipeUsers::Table)
                    .col(ColumnDef::new(RecipeUsers::Id).uuid().primary_key())
                    .col(ColumnDef::new(RecipeUsers::RecipeId).uuid().not_null())
                    .col(ColumnDef::new(RecipeUsers::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(RecipeUsers::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_tbl(RecipeUsers::Table)
                            .from_col(RecipeUsers::UserId)
                            .to_tbl(Users::Table)
                            .to_col(Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_tbl(RecipeUsers::Table)
                            .from_col(RecipeUsers::RecipeId)
                            .to_tbl(Recipes::Table)
                            .to_col(Recipes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    // Define how to rollback this migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RecipeUsers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RecipeIngredients::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Recipes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PantryItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Ingredients::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
pub enum Ingredients {
    Table,
    Id,
    Name,
    CanBeEatenRaw,
    CreatedAt,
    // InSeason,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Name,
    // Password
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum PantryItems {
    Table,
    Id,
    IngredientId,
    PurchaseDate,
    ExpirationDate,
    Quantity,
    WeightGrams,
    VolumeMilliLitres,
    UserId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum RecipeIngredients {
    Table,
    Id,
    RecipeId,
    IngredientId,
    Amount,
    Unit,
    Optional,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum RecipeUsers {
    Table,
    Id,
    RecipeId,
    UserId,
    CreatedAt,
}

#[derive(Iden)]
pub enum Recipes {
    Table,
    Id,
    Name,
    CookingTimeMins,
    Link,
    Instructions, // Equipment
    CreatedAt,
    UpdatedAt,
}
