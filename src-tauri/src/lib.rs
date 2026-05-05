pub mod models;
pub mod storage;
pub mod sync;
pub mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_todos,
            commands::add_todo,
            commands::remove_todo,
            commands::update_todo,
            commands::sync_from_cloud,
            commands::test_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use crate::models::{DateTime, Todo};
    use std::collections::HashMap;
    use uuid::Uuid;

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
        assert!(map.contains_key("autoExpire"));
        assert!(!map.contains_key("auto_expire"));
        assert_eq!(map["autoExpire"], serde_json::json!(true));
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
    }

    #[test]
    fn test_datetime_deserialize() {
        let json = r#"{"year":2024,"month":2,"day":29,"hour":0,"minute":0,"second":0}"#;
        let dt: DateTime = serde_json::from_str(json).unwrap();
        assert_eq!(dt.year, 2024);
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
        let mut updated = sample_todo();
        updated.text = "modified".into();
        updated.completed = true;
        let mut merged = vec![sample_todo()];
        for rt in vec![updated.clone()] {
            if let Some(pos) = merged.iter().position(|t| t.id == rt.id) {
                merged[pos] = rt;
            }
        }
        assert_eq!(merged[0].text, "modified");
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
