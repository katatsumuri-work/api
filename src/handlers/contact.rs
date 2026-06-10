use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 問い合わせリクエスト。
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ContactRequest {
    /// 氏名
    pub name: String,
    /// 返信先メールアドレス
    pub email: String,
    /// 問い合わせ本文
    pub message: String,
}

impl ContactRequest {
    /// 入力が受け付け可能かを判定する。
    ///
    /// 必須項目が空でなく、メールアドレスらしき形（`@` を含む）であることのみを確認する。
    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty() && self.email.contains('@') && !self.message.trim().is_empty()
    }
}

/// 問い合わせ受付ステータス。
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContactStatus {
    /// 受け付けた
    Accepted,
}

/// 問い合わせ受付レスポンス。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ContactAccepted {
    pub status: ContactStatus,
}

#[utoipa::path(
    post,
    path = "/contact",
    tag = "contact",
    request_body = ContactRequest,
    responses(
        (status = 202, description = "問い合わせを受け付けた", body = ContactAccepted),
        (status = 400, description = "入力が不正")
    )
)]
pub async fn contact(Json(payload): Json<ContactRequest>) -> Response {
    if !payload.is_valid() {
        return StatusCode::BAD_REQUEST.into_response();
    }

    // TODO: 当面はログ出力のみ。後でメール / Slack 等への転送を実装する。
    tracing::info!(
        name = %payload.name,
        email = %payload.email,
        "問い合わせを受信しました"
    );

    (
        StatusCode::ACCEPTED,
        Json(ContactAccepted {
            status: ContactStatus::Accepted,
        }),
    )
        .into_response()
}
