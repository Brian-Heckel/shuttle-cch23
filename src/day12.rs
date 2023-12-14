use std::time::SystemTime;

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use color_eyre::eyre::eyre;
use ulid::Ulid;
use uuid::Uuid;

use crate::{cch_error::ReportError, ServerState};
use serde_json::{json, Value};

#[tracing::instrument]
pub async fn save_packet(
    State(app_state): State<ServerState>,
    Path(packet): Path<String>,
) -> Result<(), ReportError> {
    app_state.add_packet(packet);
    Ok(())
}

#[tracing::instrument]
pub async fn load_packet(
    State(app_state): State<ServerState>,
    Path(packet): Path<String>,
) -> Result<String, ReportError> {
    let sec_duration = app_state
        .load_packet(packet)
        .ok_or(eyre!("Packet Hasn't been added Yet"))?;
    Ok(sec_duration.to_string())
}

#[axum::debug_handler]
#[tracing::instrument]
pub async fn convert_ulids(Json(data): Json<Value>) -> Result<Json<Value>, ReportError> {
    if let Value::Array(ulid_values) = data {
        let ulids: Vec<Ulid> = ulid_values
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        let uuids: Vec<Uuid> = ulids.into_iter().map(|ulid| ulid.into()).rev().collect();
        let json_return = serde_json::to_value(uuids).unwrap();
        Ok(Json(json_return))
    } else {
        Err(eyre!("Not an Array").into())
    }
}

#[axum::debug_handler]
#[tracing::instrument]
pub async fn ulid_info(
    Path(weekday): Path<u8>,
    Json(data): Json<Value>,
) -> Result<Json<Value>, ReportError> {
    // check if weekday is between 0 and 6
    if !(0..=6).contains(&weekday) {
        return Err(eyre!("weekday is invalid").into());
    }
    if let Value::Array(ulid_values) = data {
        let ulids: Vec<Ulid> = ulid_values
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        let made_on_eve = ulids
            .iter()
            .filter(|ulid| {
                let created_on: DateTime<Utc> = ulid.datetime().into();
                let eve = NaiveDate::from_ymd_opt(2023, 12, 24).unwrap();
                let month = created_on.month() == eve.month();
                let day = created_on.day() == eve.day();
                day && month
            })
            .count();
        let on_weekday = ulids
            .iter()
            .filter(|ulid| {
                let created_on: DateTime<Utc> = ulid.datetime().into();
                let created_weekday = created_on.weekday().number_from_monday() - 1;
                weekday == created_weekday as u8
            })
            .count();
        let futures = ulids
            .iter()
            .filter(|ulid| {
                let created_on: SystemTime = ulid.datetime();
                let now = SystemTime::now();
                created_on > now
            })
            .count();
        let one_lsb = ulids
            .iter()
            .filter(|&ulid| {
                let mask = (1u128 << 80) - 1;
                let entropy_bits = ulid.0 & mask;
                entropy_bits & 1 == 1
            })
            .count();
        let json_response = json!({
            "christmas eve": made_on_eve,
            "weekday": on_weekday,
            "in the future": futures,
            "LSB is 1": one_lsb
        });
        Ok(Json(json_response))
    } else {
        Err(eyre!("Not an Array").into())
    }
}
