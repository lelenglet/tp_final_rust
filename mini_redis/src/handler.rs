use crate::model::{Request, Response};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct Entry {
    pub value: String,
    pub expires_at: Option<Instant>,
}

pub type Store = Arc<RwLock<HashMap<String, Entry>>>;

pub async fn process_command(req: Request, store: &Store) -> Response {
    match req.cmd.as_str() {
        "PING" => Response::ok(),

        "SET" => {
            if let (Some(k), Some(v)) = (req.key, req.value) {
                let mut db = store.write().await;
                db.insert(
                    k,
                    Entry {
                        value: v.to_string().replace('"', ""),
                        expires_at: None,
                    },
                );
                Response::ok()
            } else {
                Response::error("missing key or value")
            }
        }

        "GET" => {
            if let Some(k) = req.key {
                let db = store.read().await;
                if let Some(entry) = db.get(&k) {
                    if entry.expires_at.is_none_or(|exp| exp > Instant::now()) {
                        let val = serde_json::Value::String(entry.value.clone());
                        return Response::ok_value(Some(val));
                    }
                }
                Response::ok_value(None)
            } else {
                Response::error("missing key")
            }
        }

        "KEYS" => {
            let db = store.read().await;
            let now = Instant::now();
            let keys: Vec<String> = db
                .iter()
                .filter(|(_, e)| e.expires_at.is_none_or(|exp| exp > now))
                .map(|(k, _)| k.clone())
                .collect();
            Response::ok_keys(Some(keys))
        }

        "EXPIRE" => {
            if let (Some(k), Some(s)) = (req.key, req.seconds) {
                let mut db = store.write().await;
                if let Some(entry) = db.get_mut(&k) {
                    entry.expires_at = Some(Instant::now() + Duration::from_secs(s));
                    Response::ok()
                } else {
                    Response::error("key not found")
                }
            } else {
                Response::error("missing key or seconds")
            }
        }

        "TTL" => {
            if let Some(k) = req.key {
                let db = store.read().await;
                let ttl = match db.get(&k) {
                    None => -2,
                    Some(e) => match e.expires_at {
                        None => -1,
                        Some(exp) => {
                            let now = Instant::now();
                            if exp > now {
                                (exp - now).as_secs() as i64
                            } else {
                                -2
                            }
                        }
                    },
                };
                Response::ok_ttl(Some(ttl))
            } else {
                Response::error("missing key")
            }
        }

        "DEL" => {
            if let Some(k) = req.key {
                let mut db = store.write().await;
                let count = if db.remove(&k).is_some() { 1 } else { 0 };
                Response::ok_count(Some(count))
            } else {
                Response::error("missing key")
            }
        }
        "INCR" | "DECR" => {
            if let Some(k) = req.key {
                let mut db = store.write().await;
                let entry = db.entry(k).or_insert(Entry {
                    value: "0".into(),
                    expires_at: None,
                });

                match entry.value.parse::<i64>() {
                    Ok(mut n) => {
                        if req.cmd == "INCR" {
                            n += 1;
                        } else {
                            n -= 1;
                        }
                        entry.value = n.to_string();
                        Response::ok_value(Some(serde_json::Value::Number(n.into())))
                    }
                    Err(_) => Response::error("not an integer"),
                }
            } else {
                Response::error("missing key")
            }
        }

        "SAVE" => {
            let db = store.read().await;
            let dump: HashMap<String, String> = db
                .iter()
                .map(|(k, e)| (k.clone(), e.value.clone()))
                .collect();

            match serde_json::to_string(&dump) {
                Ok(json) => {
                    if std::fs::write("dump.json", json).is_ok() {
                        Response::ok()
                    } else {
                        Response::error("could not write to file")
                    }
                }
                Err(_) => Response::error("serialization failed"),
            }
        }

        _ => Response::error("unknown command"),
    }
}
