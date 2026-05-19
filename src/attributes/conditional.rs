use serde_json::Value;
use tl::HTMLTag;
use super::{AttributeProcessor, ProcessingResult, expressions};

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
        // Delegate evaluation cleanly to our binary parser utility
        let is_truthy = expressions::evaluate_expression(attr_value, scope_stack);

        if is_truthy {
            ProcessingResult::Continue
        } else {
            ProcessingResult::SkipEntireTag
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compile_html;

    #[test]
    fn test_conditional_evaluation_truthy() {
        let template = r#"<div x-data="{ 'status': true }"><p x-if="status">Visible</p></div>"#;
        assert_eq!(compile_html(template), "<div><p>Visible</p></div>");
    }

    #[test]
    fn test_conditional_evaluation_falsy() {
        let template = r#"<div x-data="{ 'status': false }"><p x-if="status">Hidden</p></div>"#;
        assert_eq!(compile_html(template), "<div></div>");
    }

    #[test]
    fn test_binary_expression_evaluator() {
        let template = r#"<div x-data="{ 'id': 202 }"><span x-if="id === 202">Match</span><span x-if="id > 300">High</span></div>"#;
        assert_eq!(compile_html(template), "<div><span>Match</span></div>");
    }
}
