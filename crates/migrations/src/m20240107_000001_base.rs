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
                    .col(
                        ColumnDef::new(Ingredients::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Ingredients::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
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
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(
                        ColumnDef::new(Users::Admin)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
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
                    .col(ColumnDef::new(PantryItems::ExpirationDate).date())
                    .col(ColumnDef::new(PantryItems::Quantity).integer())
                    .col(ColumnDef::new(PantryItems::WeightGrams).integer())
                    .col(ColumnDef::new(PantryItems::VolumeMilliLitres).integer())
                    .col(
                        ColumnDef::new(PantryItems::Essential)
                            .boolean()
                            .default(false),
                    )
                    .col(ColumnDef::new(PantryItems::RunningLow).integer())
                    .col(ColumnDef::new(PantryItems::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(PantryItems::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(PantryItems::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
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
                    .col(ColumnDef::new(Recipes::UserId).uuid().not_null())
                    .col(ColumnDef::new(Recipes::Name).string().not_null())
                    .col(ColumnDef::new(Recipes::PrepTimeMins).integer())
                    .col(ColumnDef::new(Recipes::TotalTimeMins).integer())
                    .col(ColumnDef::new(Recipes::Link).string())
                    .col(ColumnDef::new(Recipes::Instructions).text())
                    .col(ColumnDef::new(Recipes::Image).string())
                    .col(ColumnDef::new(Recipes::LastCooked).date())
                    .col(ColumnDef::new(Recipes::Rating).integer())
                    .col(ColumnDef::new(Recipes::Notes).text())
                    .col(
                        ColumnDef::new(Recipes::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(Recipes::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_tbl(Recipes::Table)
                            .from_col(Recipes::UserId)
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
                    .table(RecipeIngredients::Table)
                    .col(
                        ColumnDef::new(RecipeIngredients::Id)
                            .uuid()
                            .unique_key()
                            .not_null(),
                    )
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
                    .col(ColumnDef::new(RecipeIngredients::Amount).string())
                    .col(ColumnDef::new(RecipeIngredients::Unit).string())
                    .col(
                        ColumnDef::new(RecipeIngredients::Optional)
                            .boolean()
                            .default(false)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RecipeIngredients::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(RecipeIngredients::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .primary_key(
                        Index::create()
                            .col(RecipeIngredients::IngredientId)
                            .col(RecipeIngredients::RecipeId),
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

        Ok(())
    }

    // Define how to rollback this migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
pub enum Ingredients {
    Table,
    Id,
    Name,
    CreatedAt,
    // InSeason,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Name,
    PasswordHash,
    Admin,
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
    Essential,
    RunningLow,
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
pub enum Recipes {
    Table,
    Id,
    UserId,
    Name,
    PrepTimeMins,
    TotalTimeMins,
    Link,
    Instructions, //TODO: Equipment, Tags (Breakfast, Lunch, Soup, Baking..)
    Image,
    LastCooked,
    Rating,
    Notes,
    CreatedAt,
    UpdatedAt,
}
