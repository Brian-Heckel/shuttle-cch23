use axum::{extract::Query, Json};
use color_eyre::eyre::eyre;
use serde::Deserialize;
use serde_json::Value;

use crate::cch_error::ReportError;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    offset: Option<usize>,
    limit: Option<usize>,
    split: Option<usize>,
}

#[tracing::instrument]
pub async fn paginate_list(
    Query(q): Query<Pagination>,
    Json(names): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ReportError> {
    let Value::Array(names) = names else {
        return Err(eyre!("No Array in Body!").into());
    };
    if names.is_empty() {
        return Ok(Json(Value::Array(names)));
    }
    let offset = q.offset.unwrap_or(0);
    let limit = match q.limit {
        Some(limit) => limit,
        None => names.len() - offset,
    };
    match q.split {
        Some(split) => {
            let paged_names = &names[offset..offset + limit];
            let split_names: Vec<_> = paged_names
                .chunks(split)
                .map(|c| Value::Array(c.to_vec()))
                .collect();
            Ok(Json(Value::Array(split_names)))
        }
        None => {
            let paged_names = &names[offset..offset + limit];
            Ok(Json(Value::Array(paged_names.to_vec())))
        }
    }
}
