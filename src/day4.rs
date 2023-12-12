use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::debug;

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
    #[serde(rename = "cAnD13s_3ATeN-yesT3rdAy")]
    candies_eaten_yesterday: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContestResults {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String,
}

#[tracing::instrument]
pub async fn reindeer_contest(
    Json(deers): Json<Vec<DeerDetailed>>,
) -> Result<Json<ContestResults>, ReportError> {
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
    let results = ContestResults {
        fastest,
        tallest,
        magician,
        consumer,
    };
    debug!(?results);
    Ok(Json(results))
}
