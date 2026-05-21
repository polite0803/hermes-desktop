use serde_json::Value as JsonValue;

/// Convert a serde_yaml::Value to serde_json::Value for path operations
pub fn yaml_to_json_value(yaml: &serde_yaml::Value) -> JsonValue {
    match yaml {
        serde_yaml::Value::Null => JsonValue::Null,
        serde_yaml::Value::Bool(b) => JsonValue::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsonValue::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                JsonValue::Number(serde_json::Number::from_f64(f).unwrap_or(0.into()))
            } else {
                JsonValue::Null
            }
        }
        serde_yaml::Value::String(s) => JsonValue::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            JsonValue::Array(seq.iter().map(yaml_to_json_value).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                let key = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    other => format!("{:?}", other),
                };
                obj.insert(key, yaml_to_json_value(v));
            }
            JsonValue::Object(obj)
        }
        _ => JsonValue::Null,
    }
}

/// Get a value by dot-separated path from a JSON value
/// e.g., "memory.provider" -> json["memory"]["provider"]
pub fn get_json_path<'a>(value: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;
    for part in parts {
        match current {
            JsonValue::Object(obj) => {
                current = obj.get(part)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// Set a value at a dot-separated path in a JSON value
pub fn set_json_path(value: &mut JsonValue, path: &str, new_val: JsonValue) {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return;
    }

    let mut current = value;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            if let JsonValue::Object(obj) = current {
                obj.insert(part.to_string(), new_val.clone());
            }
        } else {
            // Intermediate part - ensure object exists
            if let JsonValue::Object(obj) = current {
                if !obj.contains_key(*part) {
                    obj.insert(part.to_string(), JsonValue::Object(serde_json::Map::new()));
                }
                current = obj.get_mut(*part).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_simple_path() {
        let json = serde_json::json!({
            "memory": {
                "provider": "memdir"
            }
        });
        assert_eq!(
            get_json_path(&json, "memory.provider"),
            Some(&JsonValue::String("memdir".to_string()))
        );
    }

    #[test]
    fn test_get_nonexistent_path() {
        let json = serde_json::json!({"a": 1});
        assert_eq!(get_json_path(&json, "b.c"), None);
    }

    #[test]
    fn test_set_path() {
        let mut json = serde_json::json!({});
        set_json_path(&mut json, "model.default", JsonValue::String("claude-sonnet-4".into()));
        assert_eq!(
            json["model"]["default"],
            JsonValue::String("claude-sonnet-4".into())
        );
    }

    #[test]
    fn test_yaml_to_json_conversion() {
        let yaml: serde_yaml::Value = serde_yaml::from_str(r#"
memory:
  provider: memdir
models:
  - name: test
"#).unwrap();
        let json = yaml_to_json_value(&yaml);
        assert_eq!(json["memory"]["provider"], "memdir");
        assert!(json["models"].is_array());
    }
}
