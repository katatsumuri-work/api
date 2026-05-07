use axum::response::Html;

const DOCS_HTML: &str = include_str!("../assets/docs.html");

pub async fn docs() -> Html<&'static str> {
    Html(DOCS_HTML)
}
