use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Request {
    pub cmd: String,
    pub key: Option<String>,
    pub value: Option<serde_json::Value>,
    pub seconds: Option<u64>,
}

#[derive(Serialize)]
pub struct Response {
    pub status: String,
    pub value: Option<serde_json::Value>,
    pub message: Option<String>,
    pub count: Option<u32>,
    pub keys: Option<Vec<String>>,
    pub ttl: Option<i64>,
}

impl Response {
    pub fn ok() -> Self {
        Self {
            status: "ok".into(),
            value: None,
            message: None,
            count: None,
            keys: None,
            ttl: None,
        }
    }
    pub fn ok_value(v: Option<serde_json::Value>) -> Self {
        Self {
            status: "ok".into(),
            value: v,
            message: None,
            count: None,
            keys: None,
            ttl: None,
        }
    }
    pub fn ok_keys(k: Option<Vec<String>>) -> Self {
        Self {
            status: "ok".into(),
            value: None,
            message: None,
            count: None,
            keys: k.clone(),
            ttl: None,
        }
    }
    pub fn ok_ttl(t: Option<i64>) -> Self {
        Self {
            status: "ok".into(),
            value: None,
            message: None,
            count: None,
            keys: None,
            ttl: t,
        }
    }
    pub fn ok_count(c: Option<u32>) -> Self {
        Self {
            status: "ok".into(),
            value: None,
            message: None,
            count: c,
            keys: None,
            ttl: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            status: "error".into(),
            value: None,
            message: Some(msg.into()),
            count: None,
            keys: None,
            ttl: None,
        }
    }
}
