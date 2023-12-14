use axum::{extract::State, Json};
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::prelude::FromRow;

use crate::{cch_error::ReportError, ServerState};

pub async fn base_query(State(state): State<ServerState>) -> Result<String, ReportError> {
    match sqlx::query_scalar::<_, i32>("SELECT 20231213")
        .fetch_one(&state.pool)
        .await
    {
        Ok(num) => Ok(num.to_string()),
        Err(_) => Err(eyre!("Connection to db failed").into()),
    }
}

pub async fn reset_table(State(state): State<ServerState>) -> Result<(), ReportError> {
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Order {
    id: i32,
    region_id: i32,
    gift_name: String,
    quantity: i32,
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
pub async fn total_orders(
    State(state): State<ServerState>,
) -> Result<Json<serde_json::Value>, ReportError> {
    let q = "SELECT SUM(quantity) FROM orders";
    let sum = sqlx::query_scalar::<_, i64>(q)
        .fetch_one(&state.pool)
        .await
        .unwrap();
    Ok(Json(json!({"total": sum})))
}

pub async fn get_popular(
    State(state): State<ServerState>,
) -> Result<Json<serde_json::Value>, ReportError> {
    let q = r#"
SELECT gift_name, MAX(num_quantity) FROM
    (SELECT 
    gift_name,
    SUM(quantity) as num_quantity
    FROM orders
    GROUP BY gift_name
) as sub
GROUP BY gift_name
    "#;
    let query_opt = sqlx::query_as::<_, (String, i64)>(q)
        .fetch_optional(&state.pool)
        .await
        .unwrap();
    if let Some((popular_gift, _)) = query_opt {
        let json_response = json!({"popular": popular_gift});
        Ok(Json(json_response))
    } else {
        let json_response = json!({"popular": serde_json::Value::Null});
        Ok(Json(json_response))
    }
}
