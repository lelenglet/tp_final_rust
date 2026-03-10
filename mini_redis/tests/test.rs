use mini_redis::handler::{process_command, Entry, Store};
use mini_redis::model::Request;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::fs;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_full_palier_3_save() {
        let store: Store = Arc::new(RwLock::new(HashMap::new()));
        {
            let mut db = store.write().await;
            db.insert(
                "test_key".to_string(),
                Entry {
                    value: "42".to_string(),
                    expires_at: None,
                },
            );
        }

        let req_save = Request {
            cmd: "SAVE".to_string(),
            key: None,
            value: None,
            seconds: None,
        };

        let resp = process_command(req_save, &store).await;
        assert_eq!(resp.status, "ok");

        let file_content =
            fs::read_to_string("dump.json").expect("Le fichier dump.json devrait avoir été créé");

        let json: Value = serde_json::from_str(&file_content)
            .expect("Le contenu du dump doit être un JSON valide");

        assert_eq!(json["test_key"], "42");

        let _ = fs::remove_file("dump.json");
    }

    #[tokio::test]
    async fn test_incr_logic() {
        let store: Store = Arc::new(RwLock::new(HashMap::new()));

        let req_incr = Request {
            cmd: "INCR".to_string(),
            key: Some("compteur".into()),
            value: None,
            seconds: None,
        };

        let resp = process_command(req_incr, &store).await;
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.value.unwrap(), serde_json::json!(1));
    }
}
