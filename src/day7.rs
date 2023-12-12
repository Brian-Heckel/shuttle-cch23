use std::collections::HashMap;

use axum::extract::Json;
use axum_extra::extract::CookieJar;
use base64::{engine::general_purpose, Engine};
use color_eyre::eyre::{eyre, OptionExt};
use color_eyre::Report;
use serde::{Deserialize, Serialize};

use crate::cch_error::ReportError;

#[derive(Serialize, Deserialize)]
pub struct CookieRecipe {
    flour: u32,
    #[serde(rename = "chocolate chips")]
    chocolate_chips: u32,
}

#[tracing::instrument]
fn decode(bytes: &[u8]) -> Result<Vec<u8>, Report> {
    Ok(general_purpose::STANDARD.decode(bytes)?)
}

#[tracing::instrument]
fn into_recipe(message: Vec<u8>) -> Result<CookieRecipe, Report> {
    Ok(serde_json::from_slice::<CookieRecipe>(&message)?)
}

#[axum::debug_handler]
#[tracing::instrument]
pub async fn decode_recipe(jar: CookieJar) -> Result<Json<CookieRecipe>, ReportError> {
    let cookie = jar.get("recipe").ok_or_eyre("No recipe Cookie")?;
    let plain_bytes = cookie.value().as_bytes();
    let message = decode(plain_bytes)?;
    let recipe = into_recipe(message)?;
    Ok(Json(recipe))
}

#[derive(Serialize, Deserialize)]
pub struct Ingredients {
    flour: u32,
    sugar: u32,
    butter: u32,
    #[serde(rename = "baking powder")]
    baking_powder: u32,
    #[serde(rename = "chocolate chips")]
    chocolate_chips: u32,
}

#[derive(Serialize, Deserialize)]
pub struct BakeInput {
    recipe: HashMap<String, u32>,
    pantry: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize)]
pub struct BakeOutput {
    cookies: u32,
    pantry: HashMap<String, u32>,
}

#[axum::debug_handler]
#[tracing::instrument]
pub async fn bake_recipe(jar: CookieJar) -> Json<BakeOutput> {
    let cookie = jar.get("recipe").unwrap();
    let plain_bytes = cookie.value().as_bytes();
    let message = general_purpose::STANDARD.decode(plain_bytes).unwrap();
    let recipe = serde_json::from_slice::<BakeInput>(&message).unwrap();
    let amount_baked = recipe
        .recipe
        .iter()
        .map(|(&ref ingredient, &amount)| {
            let pantry_amount = recipe.pantry.get(ingredient)?;
            Some(pantry_amount / amount)
        })
        .map(|val| val.unwrap_or(0))
        .min()
        .unwrap_or(0);
    let mut new_pantry = recipe.pantry.clone();
    for (&ref ingredient, amount) in recipe.recipe.iter() {
        new_pantry
            .entry(ingredient.to_string())
            .and_modify(|total| *total = *total - (amount_baked * amount));
    }
    let output = BakeOutput {
        cookies: amount_baked,
        pantry: new_pantry,
    };
    Json(output)
}
