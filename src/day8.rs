use axum::{extract::Path, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize, Deserialize)]
pub struct PokeResponse {
    weight: f64,
}

#[axum::debug_handler]
#[tracing::instrument]
pub async fn get_pokemon_weight(Path(pokenumber): Path<u32>) -> impl IntoResponse {
    let mut base_url: String = "https://pokeapi.co/api/v2/pokemon/".into();
    base_url.push_str(&pokenumber.to_string());
    base_url.push('/');
    let body: PokeResponse = reqwest::get(base_url).await.unwrap().json().await.unwrap();
    let kilo_wieght = body.weight / 10.0;
    info!(weight = %body.weight, kilo_wieght = %kilo_wieght);
    kilo_wieght.to_string()
}

#[axum::debug_handler]
#[tracing::instrument]
pub async fn get_pokemon_momentum(Path(pokenumber): Path<u32>) -> impl IntoResponse {
    let mut base_url: String = "https://pokeapi.co/api/v2/pokemon/".into();
    base_url.push_str(&pokenumber.to_string());
    base_url.push('/');
    let g = 9.825;
    let body: PokeResponse = reqwest::get(base_url).await.unwrap().json().await.unwrap();
    let kilo_wieght = body.weight / 10.0;
    let v: f64 = 2.0 * g * 10.0;
    let p = kilo_wieght * v.sqrt();
    p.to_string()
}
