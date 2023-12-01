use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
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
        .route("/", get(hello_world));
    Ok(router.into())
}
