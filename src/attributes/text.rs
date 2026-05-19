use serde_json::Value;
use tl::HTMLTag;
use super::{AttributeProcessor, ProcessingResult};

pub struct TextProcessor;

impl AttributeProcessor for TextProcessor {
    fn name(&self) -> &'static str {
        "x-text"
    }

    fn process(
        &self,
        _tag: &HTMLTag,
        attr_value: &str,
        scope_stack: &mut Vec<Value>,
        _parser: &tl::Parser,
    ) -> ProcessingResult {
        // Run the variable environment lookup we already built
        let resolved = resolve_variable(attr_value, scope_stack);
        // We want the engine to overwrite the inner content with this resolved string
        ProcessingResult::OverrideInner(resolved)
    }
}

pub fn resolve_variable(x_text_value: &str, scope_stack: &[Value]) -> String {
    let parts: Vec<&str> = x_text_value.split('.').collect();
    let (namespace, field_key) = if parts.len() == 2 {
        (Some(parts[0]), parts[1])
    } else {
        (None, parts[0])
    };

    for scope in scope_stack.iter().rev() {
        if let Some(ns) = namespace {
            if let Some(Value::Object(map)) = scope.get(ns) {
                if let Some(val) = map.get(field_key) {
                    return format_json_value(val);
                }
            }
            if scope.is_object() && scope.get(field_key).is_some() && ns == "item" {
                if let Some(val) = scope.get(field_key) {
                    return format_json_value(val);
                }
            }
        } else if let Some(val) = scope.get(field_key) {
            return format_json_value(val);
        }
    }
    "undefined".to_string()
}

fn format_json_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::compile_html;

    #[test]
    fn test_variable_interpolation_and_scoping() {
        let template = r#"<div x-data="{ 'user': 'Alex' }"><span x-text="user">Loading</span></div>"#;
        assert_eq!(compile_html(template), "<div><span>Alex</span></div>");
    }
}
