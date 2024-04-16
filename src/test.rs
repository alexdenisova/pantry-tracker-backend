use chrono::NaiveDate;
use color_eyre::Result as AnyResult;

use crate::database::DBTrait;

pub async fn migrate_test_data(client: impl DBTrait + Send + Sync) -> AnyResult<()> {
    let chicken = client
        .create_ingredient(crate::database::ingredients::dto::CreateDto {
            name: "Chicken".to_owned(),
            can_be_eaten_raw: Some(false),
        })
        .await?;
    let rice = client
        .create_ingredient(crate::database::ingredients::dto::CreateDto {
            name: "Rice".to_owned(),
            can_be_eaten_raw: Some(false),
        })
        .await?;

    let user = client
        .create_user(crate::database::users::dto::CreateDto {
            name: "test_user".to_owned(),
        })
        .await?;
    let user_2 = client
        .create_user(crate::database::users::dto::CreateDto {
            name: "test_user_2".to_owned(),
        })
        .await?;

    client
        .create_pantry_item(crate::database::pantry_items::dto::CreateDto {
            ingredient_id: chicken.id,
            user_id: user.id,
            purchase_date: None,
            expiration_date: Some(NaiveDate::from_ymd_opt(2024, 4, 20).unwrap()),
            quantity: None,
            weight_grams: Some(400),
            volume_milli_litres: None,
        })
        .await?;
    client
        .create_pantry_item(crate::database::pantry_items::dto::CreateDto {
            ingredient_id: chicken.id,
            user_id: user_2.id,
            purchase_date: None,
            expiration_date: Some(NaiveDate::from_ymd_opt(2024, 4, 20).unwrap()),
            quantity: None,
            weight_grams: Some(400),
            volume_milli_litres: None,
        })
        .await?;
    client
        .create_pantry_item(crate::database::pantry_items::dto::CreateDto {
            ingredient_id: rice.id,
            user_id: user.id,
            purchase_date: None,
            expiration_date: None,
            quantity: None,
            weight_grams: Some(400),
            volume_milli_litres: None,
        })
        .await?;

    let chicken_recipe = client.create_recipe(crate::database::recipes::dto::CreateDto{ user_id: user.id, name: "Plain Chicken".to_owned(), cooking_time_mins: Some(20), link: None, instructions: Some("cook chicken".to_owned()), image: Some("https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcSvUbhcjwZxp2hfQGoc_ChtsN-4FF2nQ1U3yUmwEv8YSQ&s".to_owned())  }).await?;
    let chicken_recipe_2 = client.create_recipe(crate::database::recipes::dto::CreateDto{ user_id: user_2.id, name: "Plain Chicken".to_owned(), cooking_time_mins: Some(20), link: None, instructions: Some("cook chicken".to_owned()), image: Some("https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcSvUbhcjwZxp2hfQGoc_ChtsN-4FF2nQ1U3yUmwEv8YSQ&s".to_owned())  }).await?;
    let chicken_rice_recipe = client.create_recipe(crate::database::recipes::dto::CreateDto{ user_id: user.id, name: "Chicken Rice".to_owned(), cooking_time_mins: Some(30), link: Some("https://iowagirleats.com/one-pot-chicken-and-rice/".to_owned()), instructions: None, image: Some("https://static01.nyt.com/images/2023/11/14/multimedia/MB-Chicken-and-Ric-cvjf/MB-Chicken-and-Ric-cvjf-superJumbo.jpg".to_owned())  }).await?;

    client
        .create_recipe_ingredient(crate::database::recipe_ingredients::dto::CreateDto {
            recipe_id: chicken_recipe.id,
            ingredient_id: chicken.id,
            amount: Some("3".to_string()),
            unit: Some("pounds".to_owned()),
            optional: false,
        })
        .await?;
    client
        .create_recipe_ingredient(crate::database::recipe_ingredients::dto::CreateDto {
            recipe_id: chicken_recipe_2.id,
            ingredient_id: chicken.id,
            amount: Some("3".to_string()),
            unit: Some("pounds".to_owned()),
            optional: false,
        })
        .await?;
    client
        .create_recipe_ingredient(crate::database::recipe_ingredients::dto::CreateDto {
            recipe_id: chicken_rice_recipe.id,
            ingredient_id: chicken.id,
            amount: Some("3".to_string()),
            unit: Some("pounds".to_owned()),
            optional: false,
        })
        .await?;
    client
        .create_recipe_ingredient(crate::database::recipe_ingredients::dto::CreateDto {
            recipe_id: chicken_rice_recipe.id,
            ingredient_id: rice.id,
            amount: Some("1".to_string()),
            unit: Some("cup".to_owned()),
            optional: false,
        })
        .await?;

    Ok(())
}
