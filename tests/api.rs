use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use katatsumuri_api::{AppConfig, build_app};
use tower::ServiceExt;

const BODY_LIMIT_FOR_TEST: usize = 64 * 1024;

fn test_app() -> Router {
    build_app(&AppConfig::default())
}

fn cors_test_app() -> Router {
    let config = AppConfig {
        cors_allowed_origins: vec!["https://katatsumuri.work".to_string()],
        ..AppConfig::default()
    };
    build_app(&config)
}

#[tokio::test]
async fn health_エンドポイントは200と_status_okを返す() {
    let response = test_app()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), BODY_LIMIT_FOR_TEST)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn openapi_jsonは_health_を含むドキュメントを返す() {
    let response = test_app()
        .oneshot(
            Request::builder()
                .uri("/api-docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), BODY_LIMIT_FOR_TEST)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["info"]["title"], "Katatsumuri-works LLC");
    assert!(
        json["paths"]["/health"].is_object(),
        "/health が OpenAPI ドキュメントに含まれていません"
    );
}

#[tokio::test]
async fn docs_はカタツムリワークスヘッダ付きの_html_を返す() {
    let response = test_app()
        .oneshot(Request::builder().uri("/docs").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), BODY_LIMIT_FOR_TEST)
        .await
        .unwrap();
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
async fn swagger_assets_配下の_css_が配信される() {
    let response = test_app()
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

#[tokio::test]
async fn セキュリティ系のレスポンスヘッダが付与される() {
    let response = test_app()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let headers = response.headers();
    assert_eq!(
        headers.get(header::X_CONTENT_TYPE_OPTIONS).unwrap(),
        "nosniff"
    );
    assert_eq!(headers.get(header::X_FRAME_OPTIONS).unwrap(), "DENY");
    assert_eq!(
        headers.get(header::REFERRER_POLICY).unwrap(),
        "strict-origin-when-cross-origin"
    );
    assert!(headers.contains_key(header::CONTENT_SECURITY_POLICY));
    assert!(headers.contains_key(header::STRICT_TRANSPORT_SECURITY));
}

#[tokio::test]
async fn cors_allowlistが空ならcrossorigin応答ヘッダは付かない() {
    let response = test_app()
        .oneshot(
            Request::builder()
                .uri("/health")
                .header(header::ORIGIN, "https://example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        !response
            .headers()
            .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN),
        "allowlistが空のときに Access-Control-Allow-Origin が付与されています"
    );
}

#[tokio::test]
async fn cors_allowlistに含まれるoriginは許可される() {
    let response = cors_test_app()
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/health")
                .header(header::ORIGIN, "https://katatsumuri.work")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .map(|v| v.to_str().unwrap()),
        Some("https://katatsumuri.work"),
    );
}
