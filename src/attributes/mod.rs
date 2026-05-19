use serde_json::Value;
use tl::HTMLTag;

pub mod for_loop;
pub mod text;
pub mod conditional; // New module

pub use for_loop::ForLoopProcessor;
pub use text::TextProcessor;
pub use conditional::ConditionalProcessor; // Export new processor

/// The outcome of running an attribute processor on a tag node
pub enum ProcessingResult {
    /// The tag was mutated or skipped, but continue processing its original children normally
    Continue,
    /// The directive completely replaced or generated the tag's inner output (e.g., x-for). Stop standard recursive parsing.
    OverrideInner(String),
    /// The directive dictates that this entire tag and its children should be completely omitted (e.g., x-if="false")
    SkipEntireTag,
}

pub trait AttributeProcessor {
    /// Returns the directive identifier this processor looks for (e.g., "x-text")
    fn name(&self) -> &'static str;

    /// Process the tag if the attribute is present
    fn process(
        &self,
        tag: &HTMLTag,
        attr_value: &str,
        scope_stack: &mut Vec<Value>,
        parser: &tl::Parser,
    ) -> ProcessingResult;
}
