use serde_json::Value;
use tl::HTMLTag;
use super::{AttributeProcessor, ProcessingResult, expressions}; // Re-add expressions module safely

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

pub fn evaluate_and_bind_attributes(tag: &HTMLTag, scope_stack: &[Value], processors_names: &[&str]) -> String {
    let attributes = tag.attributes();
    let mut attrs_str = String::new();

    for (key, val) in attributes.iter() {
        let key_str = key.as_ref();

        if key_str.starts_with("x-bind:") || key_str.starts_with(':') {
            let target_attribute = if key_str.starts_with("x-bind:") {
                &key_str["x-bind:".len()..]
            } else {
                &key_str[":".len()..]
            };

            if let Some(v) = val {
                let expression_expr = v.as_ref();
                
                // DELEGATION UPGRADE: Run through complex evaluation engine loop cleanly!
                let resolved_str = expressions::evaluate_complex_string(expression_expr, scope_stack);

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

#[cfg(test)]
mod tests {
    use crate::compile_html;

    #[test]
    fn test_complex_attribute_template_literal_binding() {
        let template = r#"<div x-data="{ 'id': 707, 'color': '#ef4444' }"><button :id="`btn-${id}`" :style="`color: ${color};`"></button></div>"#;
        assert_eq!(compile_html(template), "<div><button id='btn-707' style='color: #ef4444;'></button></div>");
    }
}
