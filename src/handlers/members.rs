use axum::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

/// メンバー（役員・スタッフ）情報。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Member {
    /// 識別子（URL スラッグ）
    pub id: String,
    /// 氏名
    pub name: String,
    /// 役割・肩書き
    pub role: String,
    /// 自己紹介
    pub bio: String,
}

impl Member {
    /// 全メンバーを返す。
    ///
    /// 当面は 1 名（代表）想定。将来的にはデータ層へ差し替える。
    fn all() -> Vec<Member> {
        vec![Member {
            id: "yamazaki".to_string(),
            // TODO: 公開用の氏名表記に差し替える（現在はダミー）。
            name: "山崎 太郎".to_string(),
            role: "代表社員".to_string(),
            bio: "合同会社カタツムリワークスの代表。AI とソフトウェア開発を担当しています。"
                .to_string(),
        }]
    }

    /// 識別子からメンバーを 1 件取得する。該当が無ければ `None`。
    fn find(id: &str) -> Option<Member> {
        Member::all().into_iter().find(|member| member.id == id)
    }
}

#[utoipa::path(
    get,
    path = "/members",
    tag = "members",
    responses(
        (status = 200, description = "メンバー一覧を返す", body = [Member])
    )
)]
pub async fn list_members() -> Json<Vec<Member>> {
    Json(Member::all())
}

#[utoipa::path(
    get,
    path = "/members/{id}",
    tag = "members",
    params(
        ("id" = String, Path, description = "メンバーの識別子（スラッグ）")
    ),
    responses(
        (status = 200, description = "指定したメンバーを返す", body = Member),
        (status = 404, description = "メンバーが見つからない")
    )
)]
pub async fn get_member(Path(id): Path<String>) -> Response {
    match Member::find(&id) {
        Some(member) => Json(member).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
