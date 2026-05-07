use axum::Router;
use axum::response::Html;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "katatsumuri-api",
        description = "合同会社カタツムリワークスの公開 API",
        version = "0.1.0",
    ),
    tags(
        (name = "health", description = "ヘルスチェック")
    )
)]
struct ApiDoc;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct HealthResponse {
    status: String,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "サービスが稼働中であることを返す", body = HealthResponse)
    )
)]
async fn health() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse {
        status: "ok".to_string(),
    })
}

const DOCS_HTML: &str = r##"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <title>合同会社カタツムリワークス API</title>
    <link rel="stylesheet" href="/_swagger-assets/swagger-ui.css">
    <style>
        body { margin: 0; }
        .swagger-ui .topbar { display: none !important; }
        .ksw-header {
            background: #2d6e4e;
            color: #fff;
            padding: 14px 24px;
            font-family: -apple-system, BlinkMacSystemFont, "Hiragino Sans", "Yu Gothic", "Noto Sans JP", sans-serif;
            font-size: 18px;
            font-weight: 600;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
            display: flex;
            align-items: baseline;
            gap: 12px;
        }
        .ksw-header a { color: #fff; text-decoration: none; }
        .ksw-header a:hover { opacity: 0.85; }
        .ksw-header .subtitle { font-weight: 400; opacity: 0.85; font-size: 14px; }
    </style>
</head>
<body>
    <div class="ksw-header">
        <a href="https://katatsumuri.work">合同会社カタツムリワークス</a>
        <span class="subtitle">API ドキュメント</span>
    </div>
    <div id="swagger-ui"></div>
    <script src="/_swagger-assets/swagger-ui-bundle.js" charset="UTF-8"></script>
    <script src="/_swagger-assets/swagger-ui-standalone-preset.js" charset="UTF-8"></script>
    <script>
        window.onload = function () {
            window.ui = SwaggerUIBundle({
                url: "/api-docs/openapi.json",
                dom_id: "#swagger-ui",
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                layout: "StandaloneLayout"
            });
        };
    </script>
</body>
</html>"##;

async fn docs() -> Html<&'static str> {
    Html(DOCS_HTML)
}

fn build_app() -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .split_for_parts();

    let swagger_assets = SwaggerUi::new("/_swagger-assets").url("/api-docs/openapi.json", api);

    router
        .route("/docs", get(docs))
        .merge(swagger_assets)
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = build_app();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("ポート 3000 のバインドに失敗しました");

    let addr = listener
        .local_addr()
        .expect("local_addr 取得に失敗しました");
    tracing::info!(%addr, "katatsumuri-api リスニング開始");

    axum::serve(listener, app)
        .await
        .expect("サーバ起動に失敗しました");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_エンドポイントは200と_status_okを返す() {
        let app = build_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn openapi_jsonは_health_を含むドキュメントを返す() {
        let app = build_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api-docs/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["info"]["title"], "katatsumuri-api");
        assert!(
            json["paths"]["/health"].is_object(),
            "/health が OpenAPI ドキュメントに含まれていません"
        );
    }

    #[tokio::test]
    async fn docs_はカタツムリワークスヘッダ付きの_html_を返す() {
        let app = build_app();
        let response = app
            .oneshot(Request::builder().uri("/docs").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();

        assert!(
            body_str.contains("合同会社カタツムリワークス"),
            "/docs のヘッダに会社名が含まれていません"
        );
        assert!(
            body_str.contains("/_swagger-assets/swagger-ui-bundle.js"),
            "/docs から SwaggerUI のスクリプトが読み込まれていません"
        );
    }

    #[tokio::test]
    async fn _swagger_assets_配下の_css_が配信される() {
        let app = build_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/_swagger-assets/swagger-ui.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
