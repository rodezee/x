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
