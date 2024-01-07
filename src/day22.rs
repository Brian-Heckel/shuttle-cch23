use color_eyre::eyre::OptionExt;

use crate::cch_error::ReportError;

pub async fn find_no_pair(body: String) -> Result<Vec<u8>, ReportError> {
    let nums: Vec<u64> = body.lines().filter_map(|l| l.parse().ok()).collect();
    let unpaired = nums
        .into_iter()
        .reduce(|acc, e| acc ^ e)
        .ok_or_eyre("No Integers")?;
    let base = "🎁".as_bytes().repeat(unpaired as usize);
    Ok(base)
}
