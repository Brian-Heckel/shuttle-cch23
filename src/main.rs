use std::collections::HashMap;

use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use base64::{engine::general_purpose, Engine};
use pix::{chan::Channel, rgb::Rgb};
use png_pong::Decoder;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::services::ServeFile;
use tracing::Instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

enum ServerError {
    Day1NotValidPath,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ServerError::Day1NotValidPath => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Not a Valid Path!")
            }
        };
        let body = Json(json!({ "error" : error_message }));
        (status, body).into_response()
    }
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn get_error() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Oh No!")
}

#[derive(Serialize, Deserialize)]
struct Deer {
    name: String,
    strength: u32,
}

async fn reindeer_cheer(Json(deers): Json<Vec<Deer>>) -> impl IntoResponse {
    let total = deers
        .into_iter()
        .fold(0, |accum, deer| accum + deer.strength);
    total.to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct DeerDetailed {
    name: String,
    strength: u32,
    speed: f32,
    height: u32,
    antler_width: u32,
    snow_magic_power: u32,
    #[serde(rename = "cAnD13s_3ATeN-yesT3rdAy")]
    candies_eaten_yesterday: u32,
}

#[derive(Serialize, Deserialize)]
struct ContestResults {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String,
}

#[tracing::instrument]
async fn reindeer_contest(Json(deers): Json<Vec<DeerDetailed>>) -> Json<ContestResults> {
    let fastest = deers
        .iter()
        .max_by(|&x, &y| (x.speed).partial_cmp(&y.speed).unwrap())
        .unwrap()
        .name
        .clone();
    let tallest = deers.iter().max_by_key(|&x| x.height).unwrap().name.clone();
    let magician = deers
        .iter()
        .max_by_key(|&x| x.snow_magic_power)
        .unwrap()
        .name
        .clone();
    let consumer = deers
        .iter()
        .max_by_key(|&x| x.candies_eaten_yesterday)
        .unwrap()
        .name
        .clone();
    Json(ContestResults {
        fastest,
        tallest,
        magician,
        consumer,
    })
}

#[tracing::instrument]
fn recalibrate(nums: Vec<i64>) -> Result<i64, ServerError> {
    if nums.len() == 1 {
        return Ok(nums[0].pow(3));
    }
    let all_xored = nums
        .into_iter()
        .reduce(|acc, e| acc ^ e)
        .ok_or(ServerError::Day1NotValidPath)?;
    Ok(all_xored.pow(3))
}

#[axum::debug_handler]
#[tracing::instrument]
async fn recalibrate_ids(Path(path): Path<String>) -> Result<String, ServerError> {
    let nums: Vec<&str> = path.split('/').collect();
    let nums: Vec<Result<i64, _>> = nums.into_iter().map(|s| s.parse::<i64>()).collect();
    if nums.iter().any(|e| e.is_err()) {
        return Err(ServerError::Day1NotValidPath);
    }
    let nums = nums.into_iter().map(|r| r.unwrap()).collect();
    let sled = recalibrate(nums)?;
    Ok(sled.to_string())
}

#[axum::debug_handler]
#[tracing::instrument]
async fn count_elves(body: String) -> impl IntoResponse {
    let elf_count = body.matches("elf").count();
    let elf_on_a_shelf = body.matches("elf on a shelf").count();
    let just_shelf = body.matches("shelf").count() - elf_on_a_shelf;
    Json(
        json!( { "elf": elf_count, "elf on a shelf": elf_on_a_shelf, "Shelf with no elf on it": just_shelf  }),
    )
}

#[derive(Serialize, Deserialize)]
struct CookieRecipe {
    flour: u32,
    #[serde(rename = "chocolate chips")]
    chocolate_chips: u32,
}

#[axum::debug_handler]
#[tracing::instrument]
async fn decode_recipe(jar: CookieJar) -> Json<CookieRecipe> {
    let cookie = jar.get("recipe").unwrap();
    let plain_bytes = cookie.value().as_bytes();
    let message = general_purpose::STANDARD.decode(plain_bytes).unwrap();
    let recipe = serde_json::from_slice::<CookieRecipe>(&message).unwrap();
    Json(recipe)
}

#[derive(Serialize, Deserialize)]
struct Ingredients {
    flour: u32,
    sugar: u32,
    butter: u32,
    #[serde(rename = "baking powder")]
    baking_powder: u32,
    #[serde(rename = "chocolate chips")]
    chocolate_chips: u32,
}

#[derive(Serialize, Deserialize)]
struct BakeInput {
    recipe: HashMap<String, u32>,
    pantry: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize)]
struct BakeOutput {
    cookies: u32,
    pantry: HashMap<String, u32>,
}

#[axum::debug_handler]
#[tracing::instrument]
async fn bake_recipe(jar: CookieJar) -> Json<BakeOutput> {
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

#[derive(Serialize, Deserialize)]
struct PokeResponse {
    weight: u32,
}

#[axum::debug_handler]
#[tracing::instrument]
async fn get_pokemon_weight(Path(pokenumber): Path<u32>) -> impl IntoResponse {
    let mut base_url: String = "https://pokeapi.co/api/v2/pokemon/".into();
    base_url.push_str(&pokenumber.to_string());
    base_url.push_str("/");
    let body: PokeResponse = reqwest::get(base_url).await.unwrap().json().await.unwrap();
    let kilo_wieght = body.weight / 10;
    kilo_wieght.to_string()
}

#[axum::debug_handler]
#[tracing::instrument]
async fn get_pokemon_momentum(Path(pokenumber): Path<u32>) -> impl IntoResponse {
    let mut base_url: String = "https://pokeapi.co/api/v2/pokemon/".into();
    base_url.push_str(&pokenumber.to_string());
    base_url.push_str("/");
    let g = 9.825;
    let body: PokeResponse = reqwest::get(base_url).await.unwrap().json().await.unwrap();
    let kilo_wieght = (body.weight as f64) / 10.0;
    let v: f64 = 2.0 * g * 10.0;
    let p = kilo_wieght * v.sqrt();
    p.to_string()
}

#[axum::debug_handler]
#[tracing::instrument]
async fn num_red_pixels(mut multipart: Multipart) -> impl IntoResponse {
    let image = multipart.next_field().await.unwrap().unwrap();
    let data = image.text().await.unwrap();
    let decoder = Decoder::new(data.as_bytes()).unwrap().into_steps();
    let num_magic_red: usize = decoder
        .filter_map(|frame| {
            let f = frame.ok()?;
            match f.raster {
                png_pong::PngRaster::Gray8(_) => None,
                png_pong::PngRaster::Gray16(_) => None,
                png_pong::PngRaster::Rgb8(raster) => {
                    let pixels = raster.pixels();
                    let magic_red = pixels
                        .iter()
                        .filter(|&p| {
                            let r = Rgb::red(*p).to_f32();
                            let g = Rgb::green(*p).to_f32();
                            let b = Rgb::blue(*p).to_f32();
                            r > g + b
                        })
                        .count();
                    Some(magic_red)
                }
                png_pong::PngRaster::Rgb16(_) => None,
                png_pong::PngRaster::Palette(_, _, _) => None,
                png_pong::PngRaster::Graya8(_) => None,
                png_pong::PngRaster::Graya16(_) => None,
                png_pong::PngRaster::Rgba8(_) => None,
                png_pong::PngRaster::Rgba16(_) => None,
            }
        })
        .sum();
    num_magic_red.to_string()
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .init();

    let router = Router::new()
        .route("/-1/error", get(get_error))
        .route("/1/*nums", get(recalibrate_ids))
        .route("/4/strength", post(reindeer_cheer))
        .route("/4/contest", post(reindeer_contest))
        .route("/6", post(count_elves))
        .route("/7/decode", get(decode_recipe))
        .route("/7/bake", get(bake_recipe))
        .route("/8/weight/:pokenumber", get(get_pokemon_weight))
        .route("/8/drop/:pokenumber", get(get_pokemon_momentum))
        .route("/11/red_pixels", post(num_red_pixels))
        .nest_service(
            "/11/assets/decoration.png",
            ServeFile::new("assets/decoration.png"),
        )
        .route("/", get(hello_world));
    Ok(router.into())
}
