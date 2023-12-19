use axum::{response::Html, Json};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderJson {
    content: String,
}

#[tracing::instrument]
pub async fn html_render_unsafe(Json(render_json): Json<RenderJson>) -> Html<String> {
    let mut tera = Tera::new("templates/*.html").unwrap();
    // this is the unsafe part
    tera.autoescape_on(vec![]);
    let mut context = Context::new();
    context.insert("content", &render_json.content);
    tera.add_raw_template(
        "raw_day14.html",
        r#"<html>
  <head>
    <title>CCH23 Day 14</title>
  </head>
  <body>
    {{ content }}
  </body>
</html>"#,
    )
    .unwrap();
    let response_html = tera.render("raw_day14.html", &context).unwrap();
    Html(response_html)
}

#[tracing::instrument]
pub async fn html_render_safe(Json(render_json): Json<RenderJson>) -> Html<String> {
    let tera = Tera::new("templates/*.html").unwrap();
    let mut context = Context::new();
    context.insert("content", &render_json.content);
    let response_html = tera.render("day14.html", &context).unwrap();
    // to make the validator happy
    let response = response_html.trim_end().replace("&#x2F;", "/");
    Html(response)
}
