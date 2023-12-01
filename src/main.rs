use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

enum ServerError {
    Day1NotEnoughParam,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ServerError::Day1NotEnoughParam => (StatusCode::INTERNAL_SERVER_ERROR, ""),
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
        .ok_or(ServerError::Day1NotEnoughParam)?;
    Ok(all_xored.pow(3))
}

async fn recalibrate_ids(Path(ids): Path<Vec<i64>>) -> Result<String, ServerError> {
    recalibrate(ids).map(|i| i.to_string())
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/-1/error", get(get_error))
        .route("/1/:num1/:num2", get(recalibrate_ids))
        .route("/", get(hello_world));
    Ok(router.into())
}
