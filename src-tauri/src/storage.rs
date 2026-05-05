use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use uuid::Uuid;

use crate::models::Todo;

fn data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let path = app.path().app_data_dir().map_err(|e| format!("无法获取本地数据目录: {}", e))?;
    fs::create_dir_all(&path).map_err(|e| format!("无法创建数据目录: {}", e))?;
    Ok(path)
}

pub fn load_todos(app: &tauri::AppHandle) -> Result<Vec<Todo>, String> {
    let mut path = data_dir(app)?;
    path.push("todos.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let s = fs::read_to_string(&path).map_err(|e| format!("无法读取数据文件: {}", e))?;
    serde_json::from_str(&s).map_err(|e| format!("数据文件格式错误: {}", e))
}

pub fn save_todos(app: &tauri::AppHandle, todos: &[Todo]) -> Result<(), String> {
    let mut path = data_dir(app)?;
    path.push("todos.json");
    let s = serde_json::to_string_pretty(todos).map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&path, s).map_err(|e| format!("写入数据文件失败: {}", e))
}

pub fn load_deleted_ids(app: &tauri::AppHandle) -> Result<Vec<Uuid>, String> {
    let mut path = data_dir(app)?;
    path.push("deleted_ids.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let s = fs::read_to_string(&path).map_err(|e| format!("无法读取删除记录: {}", e))?;
    let ids: Vec<String> = serde_json::from_str(&s).map_err(|e| format!("删除记录格式错误: {}", e))?;
    let uuids: Vec<Uuid> = ids.iter().filter_map(|s| Uuid::parse_str(s).ok()).collect();
    Ok(uuids)
}

pub fn save_deleted_ids(app: &tauri::AppHandle, ids: &[Uuid]) -> Result<(), String> {
    let mut path = data_dir(app)?;
    path.push("deleted_ids.json");
    let s = serde_json::to_string_pretty(ids).map_err(|e| format!("序列化删除记录失败: {}", e))?;
    fs::write(&path, s).map_err(|e| format!("写入删除记录失败: {}", e))
}
