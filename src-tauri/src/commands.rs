use tauri;
use uuid::Uuid;

use crate::models::Todo;
use crate::storage;
use crate::sync;

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
        .as_millis() as u64
}

#[tauri::command]
pub fn get_todos(app: tauri::AppHandle) -> Result<Vec<Todo>, String> {
    storage::load_todos(&app)
}

#[tauri::command]
pub fn add_todo(app: tauri::AppHandle, mut todo: Todo) -> Result<(), String> {
    todo.updated_at = now_ts();
    let mut todos = storage::load_todos(&app)?;
    todos.push(todo);
    storage::save_todos(&app, &todos)
}

#[tauri::command]
pub fn remove_todo(app: tauri::AppHandle, id: Uuid) -> Result<(), String> {
    let mut todos = storage::load_todos(&app)?;
    let before = todos.len();
    todos.retain(|t| t.id != id);
    if todos.len() == before {
        return Err(format!("未找到 id={} 的待办事项", id));
    }
    storage::save_todos(&app, &todos)?;
    let mut deleted = storage::load_deleted_ids(&app)?;
    deleted.push(id);
    storage::save_deleted_ids(&app, &deleted)
}

#[tauri::command]
pub fn update_todo(app: tauri::AppHandle, mut updated: Todo) -> Result<(), String> {
    updated.updated_at = now_ts();
    let mut todos = storage::load_todos(&app)?;
    if let Some(t) = todos.iter_mut().find(|t| t.id == updated.id) {
        *t = updated;
    } else {
        return Err(format!("未找到 id={} 的待办事项", updated.id));
    }
    storage::save_todos(&app, &todos)
}

#[tauri::command]
pub async fn sync_from_cloud(app: tauri::AppHandle, server_url: String, api_token: String) -> Result<Vec<Todo>, String> {
    sync::sync_from_cloud(&app, &server_url, &api_token).await
}

#[tauri::command]
pub async fn test_connection(server_url: String, api_token: String) -> Result<String, String> {
    let base = server_url.trim_end_matches('/');
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP 客户端初始化失败: {}", e))?;

    let resp = client
        .get(format!("{}/todos", base))
        .header("X-Api-Token", api_token)
        .send()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await
        .map_err(|e| format!("服务器响应格式错误: {}", e))?;

    if let Some(err) = body.get("error") {
        return Err(format!("服务器返回错误: {}", err.as_str().unwrap_or("未知错误")));
    }

    if !status.is_success() {
        return Err(format!("服务器返回状态码: {}", status));
    }

    Ok("连接成功".to_string())
}
