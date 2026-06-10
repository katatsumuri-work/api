use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// 事業・サービス情報。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Service {
    /// 識別子（URL スラッグ）
    pub id: String,
    /// サービス名
    pub title: String,
    /// 概要
    pub description: String,
}

impl Service {
    /// 提供サービス一覧を返す。
    ///
    /// 定款の事業目的を基に主要なものを抜粋している。将来的にはデータ層へ差し替える。
    fn all() -> Vec<Service> {
        vec![
            Service {
                id: "software".to_string(),
                title: "ソフトウェア・AI 開発".to_string(),
                description: "人工知能（AI）等を用いたソフトウェア・システムの企画、研究、開発、運用、コンサルティングを行います。".to_string(),
            },
            Service {
                id: "design".to_string(),
                title: "デザイン".to_string(),
                description: "グラフィック・ウェブ・ロゴ等の各種デザインの企画、制作、コンサルティングを行います。".to_string(),
            },
            Service {
                id: "content".to_string(),
                title: "コンテンツ企画・制作".to_string(),
                description: "各種コンテンツの企画、制作、配信、サブスクリプションサービスの提供を行います。".to_string(),
            },
        ]
    }
}

#[utoipa::path(
    get,
    path = "/services",
    tag = "services",
    responses(
        (status = 200, description = "提供サービス一覧を返す", body = [Service])
    )
)]
pub async fn list_services() -> Json<Vec<Service>> {
    Json(Service::all())
}
