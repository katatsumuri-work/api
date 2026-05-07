use axum::Router;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::routing::get;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use crate::docs::docs;
use crate::handlers::health::{__path_health, health};
use crate::openapi::ApiDoc;

const CSP_VALUE: &str = "default-src 'self'; \
    script-src 'self' 'unsafe-inline'; \
    style-src 'self' 'unsafe-inline'; \
    img-src 'self' data:; \
    connect-src 'self'; \
    frame-ancestors 'none'; \
    base-uri 'self'";

const HSTS_VALUE: &str = "max-age=63072000; includeSubDomains";

pub fn build_app(config: &AppConfig) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .split_for_parts();

    let swagger_assets = SwaggerUi::new("/_swagger-assets").url("/api-docs/openapi.json", api);

    router
        .route("/docs", get(docs))
        .merge(swagger_assets)
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(CSP_VALUE),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static(HSTS_VALUE),
        ))
        .layer(cors_layer(&config.cors_allowed_origins))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            config.request_timeout,
        ))
        .layer(RequestBodyLimitLayer::new(config.request_body_limit_bytes))
        .layer(TraceLayer::new_for_http())
}

fn cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();

    let mut layer = CorsLayer::new();
    if !origins.is_empty() {
        layer = layer
            .allow_origin(origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);
    }
    layer
}
