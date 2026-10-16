//! Defensive JSON accessors matching the reference SDK's boundary behavior.
use crate::{Error, Result};
use serde_json::{Map, Value};
pub fn loads(bytes: &[u8]) -> Result<Value> {
    expect_object(serde_json::from_slice(bytes)?)
}
pub fn expect_object(value: Value) -> Result<Value> {
    if value.is_object() {
        Ok(value)
    } else {
        Err(Error::InvalidResponse)
    }
}
pub fn get_str<'a>(data: &'a Value, key: &str, default: &'a str) -> &'a str {
    data.get(key).and_then(Value::as_str).unwrap_or(default)
}
pub fn get_int(data: &Value, key: &str, default: i64) -> i64 {
    let value = &data[key];
    value
        .as_i64()
        .or_else(|| value.as_bool().map(i64::from))
        .or_else(|| value.as_str().and_then(|s| s.trim().parse().ok()))
        .unwrap_or(default)
}
pub fn get_float(data: &Value, key: &str, default: f64) -> f64 {
    let value = &data[key];
    value
        .as_f64()
        .or_else(|| value.as_bool().map(|v| if v { 1.0 } else { 0.0 }))
        .or_else(|| value.as_str().and_then(|s| s.trim().parse().ok()))
        .unwrap_or(default)
}
pub fn get_object(data: &Value, key: &str) -> Map<String, Value> {
    data.get(key)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}
