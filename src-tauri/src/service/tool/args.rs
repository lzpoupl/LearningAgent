//! 工具参数解析助手：把模型传来的 JSON 参数收敛为强类型值，并统一错误码。
//!
//! 模型倾向把 id 传成数字、把整数传成字符串，因此 id 类参数同时接受字符串与整数；
//! 类型不符或缺必填统一返回 `invalid_input`（缺必填错误码约定见工具契约）。

use serde_json::Value;

use crate::interface::error::ApiError;

/// 列表类工具的默认返回条数。
const DEFAULT_LIMIT: u32 = 50;
/// 列表类工具允许的最大返回条数，超出则钳制。
const MAX_LIMIT: u32 = 200;

fn object(args: &Value) -> Result<&serde_json::Map<String, Value>, ApiError> {
    args.as_object()
        .ok_or_else(|| ApiError::invalid_input("工具参数必须是 JSON 对象"))
}

/// 读取字段；缺失或显式为 `null` 都视为未提供。
fn get<'a>(args: &'a Value, key: &str) -> Result<Option<&'a Value>, ApiError> {
    Ok(object(args)?.get(key).filter(|value| !value.is_null()))
}

fn missing(key: &str) -> ApiError {
    ApiError::invalid_input(format!("缺少必填参数 {key}"))
}

fn type_error(key: &str, expected: &str) -> ApiError {
    ApiError::invalid_input(format!("参数 {key} 应为{expected}"))
}

/// 必填字符串。
pub fn required_str(args: &Value, key: &str) -> Result<String, ApiError> {
    match get(args, key)? {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(type_error(key, "字符串")),
        None => Err(missing(key)),
    }
}

/// 可选字符串；缺失或 `null` 返回 `None`。
pub fn optional_str(args: &Value, key: &str) -> Result<Option<String>, ApiError> {
    match get(args, key)? {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(type_error(key, "字符串")),
        None => Ok(None),
    }
}

/// 可选布尔值。
pub fn optional_bool(args: &Value, key: &str) -> Result<Option<bool>, ApiError> {
    match get(args, key)? {
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(type_error(key, "布尔值")),
        None => Ok(None),
    }
}

/// 可选非负整数。
pub fn optional_u32(args: &Value, key: &str) -> Result<Option<u32>, ApiError> {
    match get(args, key)? {
        Some(value) => match value.as_u64() {
            Some(number) if number <= u32::MAX as u64 => Ok(Some(number as u32)),
            _ => Err(type_error(key, "非负整数")),
        },
        None => Ok(None),
    }
}

/// id 取值：同时接受字符串与整数，统一返回字符串形式。
fn id_from(value: &Value, key: &str) -> Result<String, ApiError> {
    match value {
        Value::String(text) if !text.trim().is_empty() => Ok(text.clone()),
        Value::Number(number) => match (number.as_i64(), number.as_u64()) {
            (Some(signed), _) => Ok(signed.to_string()),
            (None, Some(unsigned)) => Ok(unsigned.to_string()),
            _ => Err(type_error(key, "字符串或整数")),
        },
        _ => Err(type_error(key, "字符串或整数")),
    }
}

/// 必填 id，接受字符串或整数。
pub fn required_id(args: &Value, key: &str) -> Result<String, ApiError> {
    match get(args, key)? {
        Some(value) => id_from(value, key),
        None => Err(missing(key)),
    }
}

/// 列表类工具的 `limit`：缺省 50，上限 200。
pub fn limit(args: &Value) -> Result<u32, ApiError> {
    Ok(optional_u32(args, "limit")?
        .unwrap_or(DEFAULT_LIMIT)
        .min(MAX_LIMIT))
}

/// 列表类工具的 `offset`：缺省 0。
pub fn offset(args: &Value) -> Result<u32, ApiError> {
    Ok(optional_u32(args, "offset")?.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 可选 id，接受字符串或整数。
    fn optional_id(args: &Value, key: &str) -> Result<Option<String>, ApiError> {
        match get(args, key)? {
            Some(value) => id_from(value, key).map(Some),
            None => Ok(None),
        }
    }

    #[test]
    fn required_str_enforces_presence_and_type() {
        assert_eq!(required_str(&json!({ "q": "x" }), "q").unwrap(), "x");
        assert_eq!(
            required_str(&json!({}), "q").unwrap_err().code,
            "invalid_input"
        );
        assert_eq!(
            required_str(&json!({ "q": 1 }), "q").unwrap_err().code,
            "invalid_input"
        );
        // null 等同缺失。
        assert_eq!(
            required_str(&json!({ "q": null }), "q").unwrap_err().code,
            "invalid_input"
        );
    }

    #[test]
    fn optional_helpers_accept_null_as_absent() {
        assert_eq!(optional_str(&json!({}), "q").unwrap(), None);
        assert_eq!(optional_str(&json!({ "q": null }), "q").unwrap(), None);
        assert_eq!(
            optional_bool(&json!({ "q": true }), "q").unwrap(),
            Some(true)
        );
        assert_eq!(optional_u32(&json!({ "q": 7 }), "q").unwrap(), Some(7));
        assert!(optional_u32(&json!({ "q": -1 }), "q").is_err());
        assert!(optional_u32(&json!({ "q": "7" }), "q").is_err());
    }

    #[test]
    fn id_accepts_string_and_integer() {
        assert_eq!(required_id(&json!({ "id": "12" }), "id").unwrap(), "12");
        assert_eq!(required_id(&json!({ "id": 12 }), "id").unwrap(), "12");
        assert_eq!(optional_id(&json!({}), "id").unwrap(), None);
        assert_eq!(
            optional_id(&json!({ "id": 12 }), "id").unwrap().as_deref(),
            Some("12")
        );
        assert_eq!(
            required_id(&json!({ "id": true }), "id").unwrap_err().code,
            "invalid_input"
        );
    }

    #[test]
    fn limit_defaults_and_clamps_offset_defaults() {
        assert_eq!(limit(&json!({})).unwrap(), 50);
        assert_eq!(limit(&json!({ "limit": 10 })).unwrap(), 10);
        assert_eq!(limit(&json!({ "limit": 1000 })).unwrap(), 200);
        assert_eq!(offset(&json!({})).unwrap(), 0);
        assert_eq!(offset(&json!({ "offset": 30 })).unwrap(), 30);
    }

    #[test]
    fn non_object_arguments_are_rejected() {
        assert_eq!(
            required_str(&json!("nope"), "q").unwrap_err().code,
            "invalid_input"
        );
    }
}
