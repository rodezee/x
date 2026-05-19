use serde_json::Value;
use tl::HTMLTag;

pub mod for_loop;
pub mod text;
pub mod conditional;
pub mod expressions;
pub mod bind;
pub mod include; // New module

pub use for_loop::ForLoopProcessor;
pub use text::TextProcessor;
pub use conditional::ConditionalProcessor;
pub use bind::BindProcessor;
pub use include::IncludeProcessor; // Export new processor

pub enum ProcessingResult {
    Continue,
    OverrideInner(String),
    SkipEntireTag,
}

pub trait AttributeProcessor {
    fn name(&self) -> &'static str;
    fn process(
        &self,
        tag: &HTMLTag,
        attr_value: &str,
        scope_stack: &mut Vec<Value>,
        parser: &tl::Parser,
    ) -> ProcessingResult;
}
