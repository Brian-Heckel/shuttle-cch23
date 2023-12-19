use axum::{
    extract::{Path, State},
    Json,
};
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{cch_error::ReportError, day13::Order, ServerState};

pub async fn reset_table(State(state): State<ServerState>) -> Result<(), ReportError> {
    sqlx::query("DROP TABLE IF EXISTS regions")
        .execute(&state.pool)
        .await
        .unwrap();
    sqlx::query(
        r#"
CREATE TABLE regions (
  id INT PRIMARY KEY,
  name VARCHAR(50)
);
    "#,
    )
    .execute(&state.pool)
    .await
    .unwrap();
    sqlx::query("DROP TABLE IF EXISTS orders")
        .execute(&state.pool)
        .await
        .unwrap();
    sqlx::query(
        r#"
CREATE TABLE orders (
  id INT PRIMARY KEY,
  region_id INT,
  gift_name VARCHAR(50),
  quantity INT
);
    "#,
    )
    .execute(&state.pool)
    .await
    .unwrap();
    Ok(())
}

pub async fn insert_orders(
    State(state): State<ServerState>,
    Json(data): Json<serde_json::Value>,
) -> Result<(), ReportError> {
    if let serde_json::Value::Array(unparsed_data) = data {
        let orders: Vec<Order> = unparsed_data
            .into_iter()
            .filter_map(|o| serde_json::from_value(o).ok())
            .collect();
        for order in orders {
            let insert_q =
                "INSERT INTO orders (id, region_id, gift_name, quantity) VALUES ($1, $2, $3, $4)";

            sqlx::query(insert_q)
                .bind(order.id)
                .bind(order.region_id)
                .bind(order.gift_name)
                .bind(order.quantity)
                .execute(&state.pool)
                .await
                .unwrap();
        }
        Ok(())
    } else {
        Err(eyre!("Data is not an array!").into())
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Region {
    id: i32,
    name: String,
}

pub async fn insert_regions(
    State(state): State<ServerState>,
    Json(data): Json<serde_json::Value>,
) -> Result<(), ReportError> {
    if let serde_json::Value::Array(unparsed_data) = data {
        let regions: Vec<Region> = unparsed_data
            .into_iter()
            .filter_map(|r| serde_json::from_value::<Region>(r).ok())
            .collect();
        for region in regions {
            let insert_q = "INSERT INTO regions (id, name) VALUES ($1, $2)";
            sqlx::query(insert_q)
                .bind(region.id)
                .bind(region.name)
                .execute(&state.pool)
                .await
                .unwrap();
        }
        Ok(())
    } else {
        Err(eyre!("Data is not an array!").into())
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TotalReigon {
    region: String,
    total: i64,
}

pub async fn total_per_region(
    State(state): State<ServerState>,
) -> Result<Json<Vec<TotalReigon>>, ReportError> {
    let q = r#"
SELECT name AS "region", SUM(quantity) AS "total" FROM orders
INNER JOIN regions ON regions.id = orders.region_id
GROUP BY name
ORDER BY total DESC
    "#;
    let vals = sqlx::query_as::<_, TotalReigon>(q)
        .fetch_all(&state.pool)
        .await
        .unwrap();
    Ok(Json(vals))
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TopGift {
    region: String,
    top_gifts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct RankQueryOutput {
    region_name: String,
    gift_name: String,
    quantity_rnk: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct RegionQuery {
    name: String,
}

pub async fn top_list(
    Path(number): Path<i32>,
    State(state): State<ServerState>,
) -> Result<Json<Vec<TopGift>>, ReportError> {
    let q = r#"
SELECT
    region_name,
    gift_name,
    quantity_rnk
FROM
(SELECT
    name AS region_name,
    gift_name,
    RANK() OVER (PARTITION BY name ORDER BY total_quantity DESC, gift_name ASC ) AS quantity_rnk,
    total_quantity
FROM (SELECT
        name,
        gift_name,
        SUM(quantity) AS total_quantity
FROM orders
INNER JOIN regions ON orders.region_id = regions.id
GROUP BY name, gift_name
) AS subsub) as sub
WHERE
quantity_rnk <= $1
    "#;
    let ranks = sqlx::query_as::<_, RankQueryOutput>(q)
        .bind(number)
        .fetch_all(&state.pool)
        .await
        .unwrap();

    let all_region_names = sqlx::query_as::<_, RegionQuery>("SELECT name FROM regions")
        .fetch_all(&state.pool)
        .await
        .unwrap();

    let mut output: Vec<TopGift> = all_region_names
        .into_iter()
        .map(|region_name| {
            let mut base_top_gifts: Vec<RankQueryOutput> = ranks
                .iter()
                .filter(|&rank| rank.region_name == region_name.name)
                .cloned()
                .collect();

            base_top_gifts.sort_by_key(|rank| rank.quantity_rnk);
            let top_gifts: Vec<String> = base_top_gifts
                .into_iter()
                .map(|rank| rank.gift_name)
                .collect();
            TopGift {
                region: region_name.name.clone(),
                top_gifts,
            }
        })
        .collect();
    output.sort_by_key(|tg| tg.region.clone());
    Ok(Json(output))
}
