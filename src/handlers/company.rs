use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// 会社情報。サイト（katatsumuri.work）が表示する会社概要の唯一の情報源。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Company {
    /// 商号（日本語）
    pub name: String,
    /// 商号（英語表記）
    pub legal_name: String,
    /// 設立年月日（ISO 8601, YYYY-MM-DD）
    pub founded: String,
    /// 代表社員の氏名
    pub representative: String,
    /// 本店所在地
    pub address: String,
    /// 資本金
    pub capital: String,
    /// 事業年度
    pub fiscal_year: String,
    /// 問い合わせ先メールアドレス
    pub email: String,
    /// 公式サイト URL
    pub website: String,
}

impl Company {
    /// 現在の会社情報を返す。
    ///
    /// 最初は固定値で保持する。将来的にはデータ層（D1 / KV / 外部 CMS）へ
    /// 差し替えられるよう、ハンドラとデータを分離している。
    pub fn current() -> Self {
        Self {
            name: "合同会社カタツムリワークス".to_string(),
            legal_name: "Katatsumuri Works LLC".to_string(),
            // TODO: 設立年月日の確定値（登記日）に差し替える（現在は仮置き）。
            founded: "2026-06-11".to_string(),
            representative: "山﨑 亮".to_string(),
            address: "東京都港区浜松町２丁目２番１５号　浜松町ダイヤビル２Ｆ".to_string(),
            capital: "500,000円".to_string(),
            fiscal_year: "6月1日 〜 翌年5月31日".to_string(),
            email: "info@katatsumuri.work".to_string(),
            website: "https://katatsumuri.work".to_string(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/company",
    tag = "company",
    responses(
        (status = 200, description = "会社情報を返す", body = Company)
    )
)]
pub async fn company() -> Json<Company> {
    Json(Company::current())
}
