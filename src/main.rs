use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[derive(Serialize, Deserialize)]
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

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/-1/error", get(get_error))
        .route("/1/*nums", get(recalibrate_ids))
        .route("/4/strength", post(reindeer_cheer))
        .route("/4/contest", post(reindeer_contest))
        .route("/", get(hello_world));
    Ok(router.into())
}
