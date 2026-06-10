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

/// レスポンスボディを JSON 値としてパースするテスト用ヘルパ。
async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), BODY_LIMIT_FOR_TEST)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

async fn get(uri: &str) -> axum::response::Response {
    test_app()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn company_は会社情報の_json_を返す() {
    let response = get("/company").await;
    assert_eq!(response.status(), StatusCode::OK);

    let json = json_body(response).await;
    assert_eq!(json["name"], "合同会社カタツムリワークス");
    assert_eq!(json["website"], "https://katatsumuri.work");
    // 公開ポリシー: 住所は市区まで、資本金は含めない。
    assert_eq!(json["location"], "東京都港区");
    assert!(
        json.get("capital").is_none(),
        "資本金が公開レスポンスに含まれています"
    );
}

#[tokio::test]
async fn members_はメンバー配列を返す() {
    let response = get("/members").await;
    assert_eq!(response.status(), StatusCode::OK);

    let json = json_body(response).await;
    let members = json.as_array().expect("配列ではありません");
    assert!(!members.is_empty(), "メンバーが 1 件もありません");
    assert_eq!(members[0]["id"], "yamazaki");
}

#[tokio::test]
async fn members_id_は存在するメンバーを返す() {
    let response = get("/members/yamazaki").await;
    assert_eq!(response.status(), StatusCode::OK);

    let json = json_body(response).await;
    assert_eq!(json["id"], "yamazaki");
    assert_eq!(json["role"], "代表社員");
}

#[tokio::test]
async fn members_id_は存在しない_id_で_404_を返す() {
    let response = get("/members/unknown").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn services_はサービス配列を返す() {
    let response = get("/services").await;
    assert_eq!(response.status(), StatusCode::OK);

    let json = json_body(response).await;
    let services = json.as_array().expect("配列ではありません");
    assert!(!services.is_empty(), "サービスが 1 件もありません");
}

#[tokio::test]
async fn careers_は採用情報配列を返す() {
    let response = get("/careers").await;
    assert_eq!(response.status(), StatusCode::OK);

    let json = json_body(response).await;
    assert!(json.is_array(), "配列ではありません");
}

#[tokio::test]
async fn contact_は正しい入力で_202_を返す() {
    let payload = serde_json::json!({
        "name": "問い合わせ 太郎",
        "email": "taro@example.com",
        "message": "お仕事を依頼したいです。",
    });
    let response = test_app()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/contact")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let json = json_body(response).await;
    assert_eq!(json["status"], "accepted");
}

#[tokio::test]
async fn contact_は必須項目が空なら_400_を返す() {
    let payload = serde_json::json!({
        "name": "",
        "email": "taro@example.com",
        "message": "本文",
    });
    let response = test_app()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/contact")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn openapi_jsonは新規エンドポイントを含む() {
    let response = get("/api-docs/openapi.json").await;
    assert_eq!(response.status(), StatusCode::OK);

    let json = json_body(response).await;
    for path in [
        "/company",
        "/members",
        "/members/{id}",
        "/services",
        "/careers",
        "/contact",
    ] {
        assert!(
            json["paths"][path].is_object(),
            "{path} が OpenAPI ドキュメントに含まれていません"
        );
    }
}
