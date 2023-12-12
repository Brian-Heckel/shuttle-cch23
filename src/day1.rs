use crate::cch_error::ReportError;
use axum::extract::Path;
use color_eyre::eyre::eyre;

#[tracing::instrument]
fn recalibrate(nums: Vec<i64>) -> Result<i64, ReportError> {
    if nums.len() == 1 {
        return Ok(nums[0].pow(3));
    }
    let all_xored = nums
        .into_iter()
        .reduce(|acc, e| acc ^ e)
        .ok_or(eyre!("Need at least one element"))?;
    Ok(all_xored.pow(3))
}

#[axum::debug_handler]
#[tracing::instrument]
pub async fn recalibrate_ids(Path(path): Path<String>) -> Result<String, ReportError> {
    let nums: Vec<&str> = path.split('/').collect();
    let nums: Vec<Result<i64, _>> = nums.into_iter().map(|s| s.parse::<i64>()).collect();
    if nums.iter().any(|e| e.is_err()) {
        return Err(ReportError(eyre!("Error parsing numbers")));
    }
    let nums = nums.into_iter().map(|r| r.unwrap()).collect();
    let sled = recalibrate(nums)?;
    Ok(sled.to_string())
}
