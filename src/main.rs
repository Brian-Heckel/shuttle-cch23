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
use tower::ServiceBuilder;
use tower_http::{services::ServeFile, trace::TraceLayer};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

mod cch_error;
mod day1;
mod day4;
mod day6;
mod day7;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn get_error() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Oh No!")
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
        .with(ErrorLayer::default())
        .init();

    let router = Router::new()
        .route("/-1/error", get(get_error))
        .route("/1/*nums", get(day1::recalibrate_ids))
        .route("/4/strength", post(day4::reindeer_cheer))
        .route("/4/contest", post(day4::reindeer_contest))
        .route("/6", post(day6::count_elves))
        .route("/7/decode", get(day7::decode_recipe))
        .route("/7/bake", get(day7::bake_recipe))
        .route("/8/weight/:pokenumber", get(get_pokemon_weight))
        .route("/8/drop/:pokenumber", get(get_pokemon_momentum))
        .route("/11/red_pixels", post(num_red_pixels))
        .nest_service(
            "/11/assets/decoration.png",
            ServeFile::new("assets/decoration.png"),
        )
        .route("/", get(hello_world))
        .layer(TraceLayer::new_for_http());
    Ok(router.into())
}
