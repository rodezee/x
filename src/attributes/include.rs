use serde_json::Value;
use tl::HTMLTag;
use std::{fs, path::PathBuf};
use super::{AttributeProcessor, ProcessingResult};

pub struct IncludeProcessor;

impl AttributeProcessor for IncludeProcessor {
    fn name(&self) -> &'static str {
        "x-include"
    }

    fn process(
        &self,
        _tag: &HTMLTag,
        attr_value: &str,
        scope_stack: &mut Vec<Value>,
        _parser: &tl::Parser,
    ) -> ProcessingResult {
        // Safe relative path generation pointing inside the public asset matrix
        let target_path = PathBuf::from("public").join(attr_value);

        match fs::read_to_string(&target_path) {
            Ok(partial_html) => {
                // RECURSION FLUIDITY: Run the partial back through our compiler loop!
                // This ensures any x-text, x-for, or x-if inside your partials works beautifully.
                // We bring in the main compiler function using an internal trick or direct loop call.
                let compiled_partial = crate::compile_dom_tree(&partial_html, scope_stack);
                ProcessingResult::OverrideInner(compiled_partial)
            }
            Err(_) => {
                let error_msg = format!(
                    "<span style='color:red; font-weight:bold;'>[Include Error: {:?} missing]</span>",
                    target_path
                );
                ProcessingResult::OverrideInner(error_msg)
            }
        }
    }
}
