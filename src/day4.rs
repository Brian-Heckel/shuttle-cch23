use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cch_error::ReportError;

#[derive(Serialize, Deserialize)]
pub struct Deer {
    name: String,
    strength: u32,
}

pub async fn reindeer_cheer(Json(deers): Json<Vec<Deer>>) -> impl IntoResponse {
    let total = deers
        .into_iter()
        .fold(0, |accum, deer| accum + deer.strength);
    total.to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeerDetailed {
    name: String,
    strength: u32,
    speed: f32,
    height: u32,
    antler_width: u32,
    snow_magic_power: u32,
    favorite_food: String,
    #[serde(rename = "cAnD13s_3ATeN-yesT3rdAy")]
    candies_eaten_yesterday: u32,
}

#[tracing::instrument]
pub async fn reindeer_contest(
    Json(deers): Json<Vec<DeerDetailed>>,
) -> Result<Json<serde_json::Value>, ReportError> {
    let fastest = deers
        .iter()
        .max_by(|&x, &y| (x.speed).partial_cmp(&y.speed).unwrap())
        .unwrap();
    let tallest = deers.iter().max_by_key(|&x| x.height).unwrap();
    let magician = deers.iter().max_by_key(|&x| x.snow_magic_power).unwrap();
    let consumer = deers
        .iter()
        .max_by_key(|&x| x.candies_eaten_yesterday)
        .unwrap();

    let fastest_str = format!(
        "Speeding past the finish line with a strength of {} is {}",
        fastest.strength, fastest.name
    );
    let tallest_str = format!(
        "{} is standing tall with his {} cm wide antlers",
        tallest.name, tallest.antler_width
    );
    let magician_str = format!(
        "{} could blast you away with a snow magic power of {}",
        magician.name, magician.snow_magic_power
    );
    let consumer_str = format!(
        "{} ate lots of candies, but also some {}",
        consumer.name, consumer.favorite_food
    );
    let response_json = json!({
        "fastest": fastest_str,
        "tallest": tallest_str,
        "magician": magician_str,
        "consumer": consumer_str,
        }
    );
    Ok(Json(response_json))
}
