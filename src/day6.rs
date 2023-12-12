use axum::{response::IntoResponse, Json};
use serde_json::json;
use tracing::debug;

#[axum::debug_handler]
#[tracing::instrument]
pub async fn count_elves(body: String) -> impl IntoResponse {
    let elf_count = body.matches("elf").count();
    let elf_on_a_shelf = body.matches("elf on a shelf").count();
    let just_shelf = body.matches("shelf").count() - elf_on_a_shelf;
    debug!(%elf_count, %elf_on_a_shelf, %just_shelf);
    Json(
        json!( { "elf": elf_count, "elf on a shelf": elf_on_a_shelf, "Shelf with no elf on it": just_shelf  }),
    )
}
