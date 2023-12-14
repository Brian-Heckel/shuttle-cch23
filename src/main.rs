use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use tower_http::{services::ServeFile, trace::TraceLayer};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

mod cch_error;
mod day1;
mod day11;
mod day4;
mod day6;
mod day7;
mod day8;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn get_error() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Oh No!")
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
        .route("/8/weight/:pokenumber", get(day8::get_pokemon_weight))
        .route("/8/drop/:pokenumber", get(day8::get_pokemon_momentum))
        .route("/11/red_pixels", post(day11::num_red_pixels))
        .nest_service(
            "/11/assets/decoration.png",
            ServeFile::new("assets/decoration.png"),
        )
        .route("/", get(hello_world))
        .layer(TraceLayer::new_for_http());
    Ok(router.into())
}
