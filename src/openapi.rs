use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Katatsumuri-works LLC",
        version = "0.1.0",
    ),
    tags(
        (name = "health", description = "ヘルスチェック")
    )
)]
pub struct ApiDoc;
