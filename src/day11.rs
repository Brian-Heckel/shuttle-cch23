use std::io::Cursor;

use axum::{extract::Multipart, response::IntoResponse};
use image::{io::Reader as ImageReader, ImageFormat};

#[axum::debug_handler]
#[tracing::instrument]
pub async fn num_red_pixels(mut multipart: Multipart) -> impl IntoResponse {
    let mut data = vec![];
    while let Some(field) = multipart.next_field().await.unwrap() {
        data.append(&mut field.bytes().await.unwrap().to_vec());
    }
    let reader = ImageReader::with_format(Cursor::new(data), ImageFormat::Png);
    let decoded = reader.decode().unwrap();
    let num_magic_red = decoded
        .as_rgb8()
        .unwrap()
        .pixels()
        .filter(|p| {
            let r = p[0];
            let g = p[1];
            let b = p[2];
            r > b.saturating_add(g)
        })
        .count();
    num_magic_red.to_string()
}
