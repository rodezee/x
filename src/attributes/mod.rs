use serde_json::Value;
use tl::HTMLTag;

pub mod for_loop;
pub mod text;
pub mod conditional;
pub mod expressions; // Register the expressions engine module

pub use for_loop::ForLoopProcessor;
pub use text::TextProcessor;
pub use conditional::ConditionalProcessor;

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
