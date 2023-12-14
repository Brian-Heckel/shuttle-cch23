use axum::{response::IntoResponse, Json};
use fancy_regex::Regex;
use serde_json::json;

#[axum::debug_handler]
#[tracing::instrument]
pub async fn count_elves(body: String) -> impl IntoResponse {
    let re = Regex::new(r"(?=(elf on a shelf))").unwrap();
    let elf_count = body.matches("elf").count();
    let elf_on_shelf = re.find_iter(&body).count();
    let elf_no_shelf = body.matches("shelf").count() - elf_on_shelf;
    Json(
        json!( { "elf": elf_count, "elf on a shelf": elf_on_shelf, "shelf with no elf on it": elf_no_shelf  }),
    )
}
