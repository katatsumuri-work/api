use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Katatsumuri-works LLC",
        version = "0.1.0",
    ),
    tags(
        (name = "health", description = "ヘルスチェック"),
        (name = "company", description = "会社情報"),
        (name = "members", description = "メンバー情報"),
        (name = "services", description = "事業・サービス"),
        (name = "careers", description = "採用情報"),
        (name = "contact", description = "問い合わせ")
    )
)]
pub struct ApiDoc;
