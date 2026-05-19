use serde_json::Value;
use tl::HTMLTag;
use super::{AttributeProcessor, ProcessingResult};

pub struct ForLoopProcessor;

impl AttributeProcessor for ForLoopProcessor {
    fn name(&self) -> &'static str {
        "x-for"
    }

    fn process(
        &self,
        tag: &HTMLTag,
        attr_value: &str,
        scope_stack: &mut Vec<Value>,
        parser: &tl::Parser,
    ) -> ProcessingResult {
        let inner_template = tag.inner_html(parser).to_string();
        
        let parts: Vec<&str> = attr_value.split(' ').collect();
        let array_key = parts.last().unwrap_or(&attr_value);

        let mut target_array = None;
        for scope in scope_stack.iter().rev() {
            if let Some(Value::Array(arr)) = scope.get(*array_key) {
                target_array = Some(arr.clone());
                break;
            }
        }

        if target_array.is_none() {
            if let Some(Value::Array(arr)) = scope_stack.last() {
                target_array = Some(arr.clone());
            }
        }

        let mut loop_output = String::new();
        if let Some(arr) = target_array {
            for item in arr {
                let mut iteration_context = serde_json::Map::new();
                iteration_context.insert("item".to_string(), item.clone());
                scope_stack.push(Value::Object(iteration_context));

                let rendered_nodes = crate::compile_dom_tree(&inner_template, scope_stack);
                loop_output.push_str(&rendered_nodes);

                scope_stack.pop();
            }
        } else {
            loop_output.push_str("");
        }

        // Return the full custom loop output to short-circuit the default child walker pass
        ProcessingResult::OverrideInner(loop_output)
    }
}

#[cfg(test)]
mod tests {
    use crate::compile_html;

    #[test]
    fn test_loop_iteration_unrolling() {
        let template = r#"<div x-data="[{'name':'A'}, {'name':'B'}]"><ul x-for="item"><li x-text="item.name">Tmp</li></ul></div>"#;
        assert_eq!(compile_html(template), "<div><ul><li>A</li><li>B</li></ul></div>");
    }
}
