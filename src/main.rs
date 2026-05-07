use axum::Router;
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

fn build_app() -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .split_for_parts();

    router
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", api))
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
    async fn docs_は_swaggerui_の_html_を返す() {
        let app = build_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/docs/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(
            body_str.contains("swagger-ui"),
            "/docs のレスポンスに swagger-ui が含まれていません"
        );
    }
}
