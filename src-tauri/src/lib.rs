use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct DateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: u64,
    pub text: String,
    pub completed: bool,
    pub deadline: Option<DateTime>,
    pub repeat: Option<DateTime>,
    pub repeat_rule: Option<String>,
}

fn data_file_path() -> PathBuf {
    let mut path = dirs_next::data_local_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
    path.push("countdown");
    fs::create_dir_all(&path).ok();
    path.push("todos.json");
    path
}

fn load_todos() -> Vec<Todo> {
    let path = data_file_path();
    if let Ok(s) = fs::read_to_string(&path) {
        serde_json::from_str(&s).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_todos(todos: &Vec<Todo>) {
    let path = data_file_path();
    if let Ok(s) = serde_json::to_string_pretty(todos) {
        let _ = fs::write(path, s);
    }
}

fn with_todos<F>(f: F)
where
    F: FnOnce(&mut Vec<Todo>),
{
    let mut todos = load_todos();
    f(&mut todos);
    save_todos(&todos);
}

#[tauri::command]
fn get_todos() -> Vec<Todo> {
    load_todos()
}

#[tauri::command]
fn add_todo(todo: Todo) {
    with_todos(|todos| todos.push(todo));
}

#[tauri::command]
fn remove_todo(id: u64) {
    with_todos(|todos| todos.retain(|t| t.id != id));
}

#[tauri::command]
fn update_todo(updated: Todo) {
    with_todos(|todos| {
        if let Some(t) = todos.iter_mut().find(|t| t.id == updated.id) {
            *t = updated;
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_todos, add_todo, remove_todo, update_todo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
