use std::collections::HashMap;
use serde_json::Value;
use uuid::Uuid;

use crate::models::Todo;
use crate::storage;

async fn reqwest_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP 客户端初始化失败: {}", e))
}

fn check_err(body: &Value) -> Option<String> {
    body.get("error").map(|v| v.as_str().unwrap_or("未知错误").to_string())
}

pub async fn sync_from_cloud(app: &tauri::AppHandle, server_url: &str, api_token: &str) -> Result<Vec<Todo>, String> {
    let client = reqwest_client().await?;
    let base = server_url.trim_end_matches('/');
    let local_todos = storage::load_todos(app)?;
    let deleted_ids = storage::load_deleted_ids(app)?;

    let known: HashMap<String, u64> = local_todos.iter()
        .map(|t| (t.id.to_string(), t.updated_at))
        .collect();
    let local_ts = local_todos.iter().map(|t| t.updated_at).max().unwrap_or(0);
    let delete_list: Vec<String> = deleted_ids.iter().map(|id| id.to_string()).collect();

    let sync_body = serde_json::json!({
        "known": known,
        "upsert": local_todos,
        "delete": delete_list,
        "timestamp": local_ts,
    });

    let resp = client
        .post(format!("{}/sync", base))
        .header("X-Api-Token", api_token)
        .header("Content-Type", "application/json")
        .json(&sync_body)
        .send()
        .await
        .map_err(|e| format!("同步请求失败: {}", e))?;

    let sync_resp: Value = resp.json().await.map_err(|e| format!("解析同步响应失败: {}", e))?;
    if let Some(err) = check_err(&sync_resp) { return Err(err); }

    let changes = &sync_resp["changes"];
    let remote_upsert: Vec<Todo> = serde_json::from_value(changes["upsert"].clone())
        .map_err(|e| format!("解析同步数据失败: {}", e))?;
    let remote_delete: Vec<String> = serde_json::from_value(changes["delete"].clone())
        .map_err(|e| format!("解析同步数据失败: {}", e))?;

    let local_by_id: HashMap<Uuid, Todo> = storage::load_todos(app)?
        .into_iter()
        .map(|t| (t.id, t))
        .collect();

    let delete_set: std::collections::HashSet<Uuid> = remote_delete.iter()
        .filter_map(|s| Uuid::parse_str(s).ok())
        .chain(deleted_ids.iter().copied())
        .collect();

    let mut merged: Vec<Todo> = local_by_id
        .into_iter()
        .filter(|(id, _)| !delete_set.contains(id))
        .map(|(_, t)| t)
        .collect();

    for rt in remote_upsert {
        if let Some(pos) = merged.iter().position(|t| t.id == rt.id) {
            merged[pos] = rt;
        } else {
            merged.push(rt);
        }
    }

    storage::save_todos(app, &merged)?;
    storage::save_deleted_ids(app, &[])?;
    Ok(merged)
}
