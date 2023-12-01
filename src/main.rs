use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn get_error() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Oh No!")
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/-1/error", get(get_error))
        .route("/", get(hello_world));
    Ok(router.into())
}
