use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// 採用情報（求人）。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Career {
    /// 識別子（URL スラッグ）
    pub id: String,
    /// 募集職種
    pub title: String,
    /// 雇用形態
    pub employment_type: String,
    /// 業務内容
    pub description: String,
}

impl Career {
    /// 募集中の採用情報一覧を返す。
    ///
    /// 当面はダミーの 1 件。将来的にはデータ層へ差し替える。
    fn all() -> Vec<Career> {
        // TODO: 実際の募集内容に差し替える（現在はダミー）。
        vec![Career {
            id: "software-engineer".to_string(),
            title: "ソフトウェアエンジニア".to_string(),
            employment_type: "業務委託".to_string(),
            description: "Rust / TypeScript を用いた自社プロダクト開発を担当していただきます。"
                .to_string(),
        }]
    }
}

#[utoipa::path(
    get,
    path = "/careers",
    tag = "careers",
    responses(
        (status = 200, description = "採用情報一覧を返す", body = [Career])
    )
)]
pub async fn list_careers() -> Json<Vec<Career>> {
    Json(Career::all())
}
