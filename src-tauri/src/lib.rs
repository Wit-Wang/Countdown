use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri;
use tauri::Manager;
use uuid::Uuid;

fn cloud_url() -> String {
    option_env!("CLOUD_URL").unwrap_or("http://localhost").to_string()
}
fn api_token() -> String {
    option_env!("COUNTDOWN_API_TOKEN").unwrap_or("default-token").to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: Uuid,
    pub text: String,
    pub completed: bool,
    pub deadline: Option<DateTime>,
    pub repeat: Option<DateTime>,
    pub auto_expire: bool,
    pub info: Option<String>,
    #[serde(default)]
    pub updated_at: u64,
}

fn data_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let mut path = app.path().app_data_dir().map_err(|e| format!("无法获取本地数据目录: {}", e))?;
    fs::create_dir_all(&path).map_err(|e| format!("无法创建数据目录: {}", e))?;
    path.push("todos.json");
    Ok(path)
}

fn load_todos(app: &tauri::AppHandle) -> Result<Vec<Todo>, String> {
    let path = data_file_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let s = fs::read_to_string(&path).map_err(|e| format!("无法读取数据文件: {}", e))?;
    serde_json::from_str(&s).map_err(|e| format!("数据文件格式错误: {}", e))
}

fn save_todos(app: &tauri::AppHandle, todos: &[Todo]) -> Result<(), String> {
    let path = data_file_path(app)?;
    let s = serde_json::to_string_pretty(todos).map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&path, s).map_err(|e| format!("写入数据文件失败: {}", e))
}

fn reqwest_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP 客户端初始化失败: {}", e))
}

fn check_err(body: &Value) -> Option<String> {
    body.get("error").map(|v| v.as_str().unwrap_or("未知错误").to_string())
}

#[tauri::command]
fn get_todos(app: tauri::AppHandle) -> Result<Vec<Todo>, String> {
    load_todos(&app)
}

#[tauri::command]
fn add_todo(app: tauri::AppHandle, mut todo: Todo) -> Result<(), String> {
    todo.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
        .as_millis() as u64;
    let mut todos = load_todos(&app)?;
    todos.push(todo);
    save_todos(&app, &todos)
}

#[tauri::command]
fn remove_todo(app: tauri::AppHandle, id: Uuid) -> Result<(), String> {
    let mut todos = load_todos(&app)?;
    let before = todos.len();
    todos.retain(|t| t.id != id);
    if todos.len() == before {
        return Err(format!("未找到 id={} 的待办事项", id));
    }
    save_todos(&app, &todos)
}

#[tauri::command]
fn update_todo(app: tauri::AppHandle, mut updated: Todo) -> Result<(), String> {
    updated.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
        .as_millis() as u64;
    let mut todos = load_todos(&app)?;
    if let Some(t) = todos.iter_mut().find(|t| t.id == updated.id) {
        *t = updated;
    } else {
        return Err(format!("未找到 id={} 的待办事项", updated.id));
    }
    save_todos(&app, &todos)
}

#[tauri::command]
fn sync_from_cloud(app: tauri::AppHandle) -> Result<Vec<Todo>, String> {
    let client = reqwest_client()?;
    let local_todos = load_todos(&app)?;

    // Build known: id → updatedAt
    let known: HashMap<String, u64> = local_todos.iter()
        .map(|t| (t.id.to_string(), t.updated_at))
        .collect();
    let local_ts = local_todos.iter().map(|t| t.updated_at).max().unwrap_or(0);

    // POST /sync with incremental protocol
    let sync_body = serde_json::json!({
        "known": known,
        "upsert": local_todos,
        "delete": [],
        "timestamp": local_ts,
    });
    let resp = client
        .post(format!("{}/sync", cloud_url()))
        .header("X-Api-Token", api_token())
        .header("Content-Type", "application/json")
        .json(&sync_body)
        .send()
        .map_err(|e| format!("同步请求失败: {}", e))?;
    let sync_resp: Value = resp.json().map_err(|e| format!("解析同步响应失败: {}", e))?;
    if let Some(err) = check_err(&sync_resp) { return Err(err); }

    // Parse response
    let changes = &sync_resp["changes"];
    let remote_upsert: Vec<Todo> = serde_json::from_value(changes["upsert"].clone())
        .map_err(|e| format!("解析同步数据失败: {}", e))?;
    let remote_delete: Vec<String> = serde_json::from_value(changes["delete"].clone())
        .map_err(|e| format!("解析同步数据失败: {}", e))?;

    // Apply changes locally
    let mut merged = load_todos(&app)?;
    let delete_ids: std::collections::HashSet<Uuid> = remote_delete.iter()
        .filter_map(|s| Uuid::parse_str(s).ok())
        .collect();
    merged.retain(|t| !delete_ids.contains(&t.id));
    for rt in remote_upsert {
        if let Some(pos) = merged.iter().position(|t| t.id == rt.id) {
            merged[pos] = rt;
        } else {
            merged.push(rt);
        }
    }

    save_todos(&app, &merged)?;
    Ok(merged)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_todos,
            add_todo,
            remove_todo,
            update_todo,
            sync_from_cloud,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    fn sample_dt() -> DateTime {
        DateTime { year: 2026, month: 5, day: 4, hour: 14, minute: 30, second: 0 }
    }

    fn sample_todo() -> Todo {
        Todo {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            text: "test".into(),
            completed: false,
            deadline: Some(sample_dt()),
            repeat: None,
            auto_expire: true,
            info: Some("notes".into()),
            updated_at: 1714800000000,
        }
    }

    #[test]
    fn test_todo_serialize_round_trip() {
        let todo = sample_todo();
        let json = serde_json::to_string(&todo).unwrap();
        let back: Todo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, todo.id);
        assert_eq!(back.text, "test");
        assert_eq!(back.auto_expire, true);
        assert_eq!(back.info, Some("notes".into()));
    }

    #[test]
    fn test_todo_camel_case_keys() {
        let todo = sample_todo();
        let json = serde_json::to_value(&todo).unwrap();
        let map = json.as_object().unwrap();

        assert!(map.contains_key("id"));
        assert!(map.contains_key("autoExpire"), "expected camelCase 'autoExpire'");
        assert!(!map.contains_key("auto_expire"), "snake_case 'auto_expire' should NOT appear");

        assert_eq!(map["autoExpire"], serde_json::json!(true));
        assert_eq!(map["text"], "test");
    }

    #[test]
    fn test_todo_updated_at_defaults_to_zero() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "text": "no-ts",
            "completed": false,
            "deadline": null,
            "repeat": null,
            "autoExpire": false,
            "info": null
        }"#;
        let todo: Todo = serde_json::from_str(json).unwrap();
        assert_eq!(todo.updated_at, 0);
    }

    #[test]
    fn test_todo_missing_info_is_none() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "text": "no-info",
            "completed": false,
            "deadline": null,
            "repeat": null,
            "autoExpire": false
        }"#;
        let todo: Todo = serde_json::from_str(json).unwrap();
        assert_eq!(todo.info, None);
    }

    #[test]
    fn test_datetime_serialize_fields() {
        let dt = sample_dt();
        let json = serde_json::to_value(&dt).unwrap();
        let map = json.as_object().unwrap();
        assert_eq!(map["year"], 2026);
        assert_eq!(map["month"], 5);
        assert_eq!(map["day"], 4);
        assert_eq!(map["hour"], 14);
        assert_eq!(map["minute"], 30);
        assert_eq!(map["second"], 0);
    }

    #[test]
    fn test_datetime_deserialize() {
        let json = r#"{"year":2024,"month":2,"day":29,"hour":0,"minute":0,"second":0}"#;
        let dt: DateTime = serde_json::from_str(json).unwrap();
        assert_eq!(dt.year, 2024);
        assert_eq!(dt.month, 2);
        assert_eq!(dt.day, 29);
    }

    #[test]
    fn test_known_map_structure() {
        let todos = vec![
            sample_todo(),
            Todo {
                id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap(),
                text: "another".into(),
                completed: true,
                deadline: None,
                repeat: None,
                auto_expire: false,
                info: None,
                updated_at: 100,
            },
        ];
        let known: HashMap<String, u64> = todos.iter()
            .map(|t| (t.id.to_string(), t.updated_at))
            .collect();
        assert_eq!(known.len(), 2);
        assert_eq!(known["550e8400-e29b-41d4-a716-446655440000"], 1714800000000);
        assert_eq!(known["660e8400-e29b-41d4-a716-446655440001"], 100);
    }

    #[test]
    fn test_merge_upsert_new() {
        let existing = vec![sample_todo()];
        let incoming = vec![
            Todo {
                id: Uuid::parse_str("770e8400-e29b-41d4-a716-446655440002").unwrap(),
                text: "new".into(),
                completed: false,
                deadline: None,
                repeat: None,
                auto_expire: false,
                info: None,
                updated_at: 0,
            },
        ];
        let mut merged = existing;
        for rt in incoming {
            if let Some(pos) = merged.iter().position(|t| t.id == rt.id) {
                merged[pos] = rt;
            } else {
                merged.push(rt);
            }
        }
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_upsert_overwrite() {
        let original = sample_todo();
        let mut updated = sample_todo();
        updated.text = "modified".into();
        updated.completed = true;

        let mut merged = vec![original];
        for rt in vec![updated.clone()] {
            if let Some(pos) = merged.iter().position(|t| t.id == rt.id) {
                merged[pos] = rt;
            }
        }
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].text, "modified");
        assert_eq!(merged[0].completed, true);
    }

    #[test]
    fn test_merge_delete() {
        let todos = vec![sample_todo()];
        let delete_ids: std::collections::HashSet<Uuid> = [
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        ].into();
        let merged: Vec<Todo> = todos.into_iter().filter(|t| !delete_ids.contains(&t.id)).collect();
        assert!(merged.is_empty());
    }
}
