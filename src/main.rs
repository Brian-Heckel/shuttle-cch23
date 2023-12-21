use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use day19::BirdState;
use sqlx::PgPool;
use tower_http::{services::ServeFile, trace::TraceLayer};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

mod cch_error;
mod day1;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day18;
mod day19;
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

#[derive(Debug, Clone)]
struct ServerState {
    pool: PgPool,
    packet_map: Arc<Mutex<HashMap<String, i64>>>,
    bird_state: Arc<BirdState>,
}

impl ServerState {
    pub fn add_packet(&self, packet: String) {
        self.packet_map
            .lock()
            .unwrap()
            .insert(packet, DateTime::timestamp(&Utc::now()));
    }

    pub fn load_packet(&self, packet: String) -> Option<i64> {
        let map = self.packet_map.lock().unwrap();
        let start = map.get(&packet)?;
        Some(DateTime::timestamp(&Utc::now()) - start)
    }
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres()] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    color_eyre::install().unwrap();

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(ErrorLayer::default())
        .init();

    let state = ServerState {
        pool,
        packet_map: Arc::new(Mutex::new(HashMap::new())),
        bird_state: Default::default(),
    };

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
        .route("/12/save/:packet", post(day12::save_packet))
        .route("/12/load/:packet", get(day12::load_packet))
        .route("/12/ulids", post(day12::convert_ulids))
        .route("/12/ulids/:weekday", post(day12::ulid_info))
        .route("/13/sql", get(day13::base_query))
        .route("/13/reset", post(day13::reset_table))
        .route("/13/orders", post(day13::insert_orders))
        .route("/13/orders/total", get(day13::total_orders))
        .route("/13/orders/popular", get(day13::get_popular))
        .route("/14/unsafe", post(day14::html_render_unsafe))
        .route("/14/safe", post(day14::html_render_safe))
        .route("/15/nice", post(day15::nice))
        .route("/15/game", post(day15::game))
        .route("/18/reset", post(day18::reset_table))
        .route("/18/regions", post(day18::insert_regions))
        .route("/18/orders", post(day18::insert_orders))
        .route("/18/regions/total", get(day18::total_per_region))
        .route("/18/regions/top_list/:number", get(day18::top_list))
        .route("/19/ws/ping", get(day19::ready_game))
        .route("/19/reset", post(day19::reset_tweet_count))
        .route("/19/views", get(day19::get_tweet_count))
        .route(
            "/19/ws/room/:room_number/user/:user_name",
            get(day19::connect_room),
        )
        .nest_service(
            "/11/assets/decoration.png",
            ServeFile::new("assets/decoration.png"),
        )
        .route("/", get(hello_world))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    Ok(router.into())
}
