use std::collections::HashMap;

use axum::extract::Json;
use axum_extra::extract::CookieJar;
use base64::{engine::general_purpose, Engine};
use color_eyre::eyre::{eyre, OptionExt};
use color_eyre::Report;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tracing::{debug, info};

use crate::cch_error::ReportError;

#[derive(Serialize, Deserialize)]
#[serde(rename = "snake_case")]
pub struct Recipe {
    recipe: HashMap<String, u32>,
}

fn decode(bytes: &[u8]) -> Result<Vec<u8>, Report> {
    Ok(general_purpose::STANDARD.decode(bytes)?)
}

fn into_recipe(message: Vec<u8>) -> Result<Recipe, Report> {
    Ok(serde_json::from_slice::<Recipe>(&message)?)
}

#[axum::debug_handler]
#[tracing::instrument]
pub async fn decode_recipe(jar: CookieJar) -> Result<Json<Recipe>, ReportError> {
    let cookie = jar.get("recipe").ok_or_eyre("No recipe Cookie")?;
    let plain_bytes = cookie.value().as_bytes();
    let message = decode(plain_bytes)?;
    let recipe = into_recipe(message)?;
    Ok(Json(recipe))
}

#[derive(Serialize, Deserialize)]
pub struct Ingredients {
    flour: u64,
    sugar: u64,
    butter: u64,
    #[serde(rename = "baking powder")]
    baking_powder: u64,
    #[serde(rename = "chocolate chips")]
    chocolate_chips: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BakeInput {
    recipe: HashMap<String, u64>,
    pantry: HashMap<String, u64>,
}

impl BakeInput {
    fn find_amount_baked(&self) -> u64 {
        self.recipe
            .iter()
            .map(|(ingredient, amount)| {
                if *amount == 0 {
                    return Some(u64::MAX);
                }
                let pantry_amount = self.pantry.get(ingredient)?;
                pantry_amount.checked_div(*amount)
            })
            .map(|val| val.unwrap_or(0))
            .min()
            .unwrap_or(0)
    }

    /// Finds the items in the recipe that is not in the
    /// pantry and then adds them in the pantry to make it 0
    pub fn adjust_pantry(&mut self) {
        for key in self.recipe.keys().cloned() {
            self.pantry.entry(key).or_insert(0);
        }
    }

    /// assumes the pantry is adjusted
    pub fn bake(&self) -> BakeOutput {
        let cookies = self.find_amount_baked();
        let new_pantry: HashMap<String, u64> = self
            .pantry
            .iter()
            .filter_map(|(ingredient, stock)| match self.recipe.get(ingredient) {
                Some(recipe_amount) => {
                    let new_stock: u64 = stock - (cookies * recipe_amount);
                    Some((ingredient.clone(), new_stock))
                }
                None => Some((ingredient.clone(), *stock)),
            })
            .collect();
        BakeOutput {
            cookies,
            pantry: new_pantry,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BakeOutput {
    cookies: u64,
    pantry: HashMap<String, u64>,
}

#[axum::debug_handler]
#[tracing::instrument]
pub async fn bake_recipe(jar: CookieJar) -> Json<BakeOutput> {
    let cookie = jar.get("recipe").unwrap();
    let plain_bytes = cookie.value().as_bytes();
    let message = general_purpose::STANDARD.decode(plain_bytes).unwrap();
    let bake_input = serde_json::from_slice::<BakeInput>(&message).unwrap();
    info!(message = ?bake_input);
    let output = bake_input.bake();
    Json(output)
}
