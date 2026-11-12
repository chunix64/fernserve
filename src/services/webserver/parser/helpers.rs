use alloc::string::{String, ToString};

use anyhow::{Context, Result};
use serde_json::{Value, json};

use super::WebServerParser;

impl WebServerParser {
    pub fn parse_param_key(query: &str, key: &str) -> Result<Option<String>> {
        let query = query.strip_prefix('?').unwrap_or(query);
        Ok(form_urlencoded::parse(query.as_bytes())
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.into_owned()))
    }

    pub fn parse_json_key(body: &str, key: &str) -> Result<String> {
        let value: Value = serde_json::from_str(body)?;

        Ok(value[key]
            .as_str()
            .context("Parse JSON key failed")?
            .to_string())
    }

    pub fn parse_json(status_code: u16, key: &str, value: &str) -> String {
        let json = json!({
            "status": status_code,
            key: value
        });

        json.to_string()
    }

    pub fn parse_error_json(status_code: u16, error: &str) -> String {
        let json = json!({
            "status": status_code,
            "error": error
        });

        json.to_string()
    }
}
