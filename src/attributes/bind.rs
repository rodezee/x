use serde_json::Value;
use tl::HTMLTag;
use super::{AttributeProcessor, ProcessingResult};

pub struct BindProcessor;

impl AttributeProcessor for BindProcessor {
    fn name(&self) -> &'static str {
        "x-bind"
    }

    fn process(
        &self,
        _tag: &HTMLTag,
        _attr_value: &str,
        _scope_stack: &mut Vec<Value>,
        _parser: &tl::Parser,
    ) -> ProcessingResult {
        ProcessingResult::Continue
    }
}

/// Helper to execute binding evaluation values dynamically onto the target element attribute string buffer
pub fn evaluate_and_bind_attributes(tag: &HTMLTag, scope_stack: &[Value], processors_names: &[&str]) -> String {
    let attributes = tag.attributes();
    let mut attrs_str = String::new();

    for (key, val) in attributes.iter() {
        // FIX: key is already a Cow<str>, so we can just call .as_ref() directly!
        let key_str = key.as_ref();

        if key_str.starts_with("x-bind:") || key_str.starts_with(':') {
            let target_attribute = if key_str.starts_with("x-bind:") {
                &key_str["x-bind:".len()..]
            } else {
                &key_str[":".len()..]
            };

            if let Some(v) = val {
                let expression_expr = v.as_ref();
                let parts: Vec<&str> = expression_expr.split('.').collect();
                let (namespace, field_key) = if parts.len() == 2 { (Some(parts[0]), parts[1]) } else { (None, parts[0]) };

                let mut resolved_str = "undefined".to_string();
                for scope in scope_stack.iter().rev() {
                    let found_val = if let Some(ns) = namespace {
                        if let Some(Value::Object(map)) = scope.get(ns) { map.get(field_key) }
                        else if scope.is_object() && scope.get(field_key).is_some() && ns == "item" { scope.get(field_key) }
                        else { None }
                    } else {
                        scope.get(field_key)
                    };

                    if let Some(json_val) = found_val {
                        resolved_str = match json_val {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => json_val.to_string(),
                        };
                        break;
                    }
                }

                attrs_str.push_str(&format!(" {}='{}'", target_attribute, resolved_str));
            }
        } else {
            if key_str != "x-data" && !processors_names.contains(&key_str) {
                if let Some(v) = val {
                    attrs_str.push_str(&format!(" {}='{}'", key_str, v.as_ref()));
                } else {
                    attrs_str.push_str(&format!(" {}", key_str));
                }
            }
        }
    }

    attrs_str
}
