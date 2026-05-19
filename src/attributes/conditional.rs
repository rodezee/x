use serde_json::Value;
use tl::HTMLTag;
use super::{AttributeProcessor, ProcessingResult};

pub struct ConditionalProcessor;

impl AttributeProcessor for ConditionalProcessor {
    fn name(&self) -> &'static str {
        "x-if"
    }

    fn process(
        &self,
        _tag: &HTMLTag,
        attr_value: &str,
        scope_stack: &mut Vec<Value>,
        _parser: &tl::Parser,
    ) -> ProcessingResult {
        // Resolve the variable expression using our existing scope stack logic
        let is_truthy = eval_truthiness(attr_value, scope_stack);

        if is_truthy {
            // The condition passed! Let the engine continue rendering this tag and its children normally
            ProcessingResult::Continue
        } else {
            // Condition failed. Tell the compiler to completely drop this tag from existence
            ProcessingResult::SkipEntireTag
        }
    }
}

/// Simple truthiness evaluator against the scope stack
fn eval_truthiness(expression: &str, scope_stack: &[Value]) -> bool {
    let parts: Vec<&str> = expression.split('.').collect();
    let (namespace, field_key) = if parts.len() == 2 {
        (Some(parts[0]), parts[1])
    } else {
        (None, parts[0])
    };

    for scope in scope_stack.iter().rev() {
        let target_val = if let Some(ns) = namespace {
            if let Some(Value::Object(map)) = scope.get(ns) {
                map.get(field_key)
            } else if scope.is_object() && scope.get(field_key).is_some() && ns == "item" {
                scope.get(field_key)
            } else {
                None
            }
        } else {
            scope.get(field_key)
        };

        if let Some(val) = target_val {
            return match val {
                Value::Bool(b) => *b,
                Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                Value::String(s) => !s.is_empty() && s != "false",
                Value::Array(arr) => !arr.is_empty(),
                Value::Object(_) => true,
                Value::Null => false,
            };
        }
    }

    // Default to false if the variable doesn't exist anywhere in the scopes
    false
}
