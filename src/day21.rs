use std::time::Duration;

use axum::extract::{Path, State};
use color_eyre::eyre::{eyre, OptionExt};
use dms_coordinates::DMS3d;
use isocountry::{CountryCode, CountryCodeParseErr};
use reqwest::ClientBuilder;
use s2::{cellid::CellID, point::Point};
use serde::Deserialize;
use tracing::info;

use crate::{cch_error::ReportError, ServerState};

pub async fn get_cell(Path(binary): Path<String>) -> Result<String, ReportError> {
    let bin = u64::from_str_radix(binary.as_ref(), 2).map_err(|_| eyre!("Not a valid binary"))?;
    let cell_id = CellID(bin);
    let point = Point(cell_id.raw_point());
    let lat = point.latitude().deg();
    let long = point.longitude().deg();
    let dms = DMS3d::from_decimal_degrees(lat, long, None);
    let lat = dms.latitude;
    let long = dms.longitude;
    let lat_bearing = match lat.bearing {
        dms_coordinates::Bearing::North => Some('N'),
        dms_coordinates::Bearing::South => Some('S'),
        dms_coordinates::Bearing::West => Some('W'),
        dms_coordinates::Bearing::East => Some('E'),
        dms_coordinates::Bearing::NorthEast => None,
        dms_coordinates::Bearing::SouthEast => None,
        dms_coordinates::Bearing::NorthWest => None,
        dms_coordinates::Bearing::SouthWest => None,
    }
    .ok_or_eyre("Got lat bearing of diagonal direction")?;
    let long_bearing = match long.bearing {
        dms_coordinates::Bearing::North => Some('N'),
        dms_coordinates::Bearing::South => Some('S'),
        dms_coordinates::Bearing::West => Some('W'),
        dms_coordinates::Bearing::East => Some('E'),
        dms_coordinates::Bearing::NorthEast => None,
        dms_coordinates::Bearing::SouthEast => None,
        dms_coordinates::Bearing::NorthWest => None,
        dms_coordinates::Bearing::SouthWest => None,
    }
    .ok_or_eyre("Got lat bearing of diagonal direction")?;
    let long = dms.longitude;
    Ok(format!(
        "{0}°{1}\'{2:.3}\'\'{3} {4}°{5}\'{6:.3}\'\'{7}",
        lat.degrees,
        lat.minutes,
        lat.seconds,
        lat_bearing,
        long.degrees,
        long.minutes,
        long.seconds,
        long_bearing
    ))
}

#[derive(Debug, Deserialize)]
struct OsmResponse {
    address: OsmAddress,
}

#[derive(Debug, Deserialize)]
struct OsmAddress {
    country_code: String,
}

pub async fn get_country(
    Path(binary): Path<String>,
    State(state): State<ServerState>,
) -> Result<String, ReportError> {
    let bin = u64::from_str_radix(binary.as_ref(), 2).map_err(|_| eyre!("Not a valid binary"))?;
    let cell_id = CellID(bin);
    let point = Point(cell_id.raw_point());
    let lat = point.latitude().deg();
    let long = point.longitude().deg();

    let api_url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=jsonv2&lat={}&lon={}",
        lat, long
    );
    info!(%api_url);

    let _permit = state.one_second_request_lock.acquire().await?;

    let client = ClientBuilder::new()
        .user_agent("cch23-shuttle/1.0.0")
        .build()?;
    let response = client.get(&api_url).send().await?;
    let osm_response: OsmResponse = response.json().await?;
    // we need this to make sure we follow the terms of the
    // open street map terms and service
    tokio::time::sleep(Duration::from_secs(2)).await;
    let raw_code = osm_response.address.country_code.to_uppercase();
    let country_code = match osm_response.address.country_code.len() {
        2 => CountryCode::for_alpha2(raw_code.as_ref()),
        3 => CountryCode::for_alpha3(raw_code.as_ref()),
        _ => Err(CountryCodeParseErr::InvalidID { unknown: 0 }),
    }?;
    let country_str = {
        if let CountryCode::BRN = country_code {
            "Brunei".to_string()
        } else {
            country_code.name().to_string()
        }
    };
    Ok(country_str)
}
