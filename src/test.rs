use chrono::NaiveDate;
use color_eyre::Result as AnyResult;

use crate::database::DBTrait;
use crate::server::routes::utils::hash_password;

#[allow(clippy::too_many_lines)]
pub async fn migrate_test_data(client: impl DBTrait + Send + Sync) -> AnyResult<()> {
    let chicken = client
        .create_ingredient(crate::database::ingredients::dto::CreateDto {
            name: "Chicken".to_owned(),
        })
        .await?;
    let rice = client
        .create_ingredient(crate::database::ingredients::dto::CreateDto {
            name: "Rice".to_owned(),
        })
        .await?;
    let _garlic = client
        .create_ingredient(crate::database::ingredients::dto::CreateDto {
            name: "Garlic".to_owned(),
        })
        .await?;

    let user = client
        .create_user(crate::database::users::dto::CreateDto {
            name: "demo".to_owned(),
            password_hash: hash_password(""),
            admin: Some(false),
        })
        .await?;
    let admin = client
        .create_user(crate::database::users::dto::CreateDto {
            name: "admin".to_owned(),
            password_hash: hash_password("2345"),
            admin: Some(true),
        })
        .await?;

    client
        .create_pantry_item(crate::database::pantry_items::dto::CreateDto {
            ingredient_id: chicken.id,
            user_id: user.id,
            expiration_date: Some(NaiveDate::from_ymd_opt(2024, 4, 20).unwrap()),
            quantity: None,
            weight_grams: Some(400),
            volume_milli_litres: None,
            essential: false,
            running_low: None,
        })
        .await?;
    client
        .create_pantry_item(crate::database::pantry_items::dto::CreateDto {
            ingredient_id: chicken.id,
            user_id: admin.id,
            expiration_date: Some(NaiveDate::from_ymd_opt(2024, 4, 20).unwrap()),
            quantity: None,
            weight_grams: Some(400),
            volume_milli_litres: None,
            essential: true,
            running_low: None,
        })
        .await?;
    client
        .create_pantry_item(crate::database::pantry_items::dto::CreateDto {
            ingredient_id: rice.id,
            user_id: user.id,
            expiration_date: None,
            quantity: None,
            weight_grams: Some(400),
            volume_milli_litres: None,
            essential: true,
            running_low: Some(500),
        })
        .await?;

    let chicken_recipe = client.create_recipe(crate::database::recipes::dto::CreateDto{
        user_id: user.id,
        name: "Plain Chicken".to_owned(),
        total_time_mins: Some(20),
        link: None,
        instructions: Some("cook chicken".to_owned()),
        image: Some("https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcSvUbhcjwZxp2hfQGoc_ChtsN-4FF2nQ1U3yUmwEv8YSQ&s".to_owned()),
        prep_time_mins: Some(5),
        last_cooked: NaiveDate::from_ymd_opt(2024, 5, 4),
        rating: Some(5),
        notes: Some("add more salt".to_owned()) 
    }).await?;
    let chicken_recipe_2 = client.create_recipe(crate::database::recipes::dto::CreateDto{
        user_id: admin.id,
        name: "Plain Chicken".to_owned(),
        total_time_mins: Some(20),
        link: None,
        instructions: Some("cook chicken".to_owned()),
        image: Some("https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcSvUbhcjwZxp2hfQGoc_ChtsN-4FF2nQ1U3yUmwEv8YSQ&s".to_owned()),
        prep_time_mins: Some(5),
        last_cooked: NaiveDate::from_ymd_opt(2024, 5, 4),
        rating: Some(5),
        notes: Some("add more salt".to_owned()) 
    }).await?;
    let chicken_rice_recipe = client.create_recipe(crate::database::recipes::dto::CreateDto{
        user_id: user.id,
        name: "Chicken Rice".to_owned(),
        total_time_mins: Some(30),
        link: Some("https://iowagirleats.com/one-pot-chicken-and-rice/".to_owned()),
        instructions: None,
        image: Some("https://static01.nyt.com/images/2023/11/14/multimedia/MB-Chicken-and-Ric-cvjf/MB-Chicken-and-Ric-cvjf-superJumbo.jpg".to_owned()),
        prep_time_mins: Some(5),
        last_cooked: NaiveDate::from_ymd_opt(2024, 4, 24),
        rating: Some(3),
        notes: None
    }).await?;

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

    let dinner_category = client
        .create_category(crate::database::categories::dto::CreateDto {
            name: "Dinner".to_owned(),
        })
        .await?;
    client
        .create_recipe_category(crate::database::recipe_categories::dto::CreateDto {
            recipe_id: chicken_rice_recipe.id,
            category_id: dinner_category.id,
        })
        .await?;

    Ok(())
}
