use serde_json::{json, Value};
use tl::HTMLTag;
use super::{AttributeProcessor, ProcessingResult, expressions};

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
        let attr_value = attr_value.trim();

        // 1. Unpack Alpine.js standard "LHS in RHS" syntax structure
        let (iterator_expr, source_expr) = if let Some(idx) = attr_value.find(" in ") {
            (attr_value[..idx].trim(), attr_value[idx + 4..].trim())
        } else {
            // Fallback backward compatibility wrapper for old raw x-for="item" syntax
            ("item", attr_value)
        };

        // 2. Parse out custom variable names (supporting index injection rules)
        let (item_var_name, index_var_name) = if iterator_expr.starts_with('(') && iterator_expr.ends_with(')') {
            let inner = &iterator_expr[1..iterator_expr.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 2 {
                (parts[0], Some(parts[1]))
            } else {
                (parts[0], None)
            }
        } else {
            (iterator_expr, None)
        };

        // 3. Resolve iterable target elements (Array vs Numeric Range)
        let iterations: Vec<Value> = if let Ok(range_max) = source_expr.parse::<i64>() {
            // Support x-for="i in 10" numeric loops
            (1..=range_max).map(Value::from).collect()
        } else {
            // Standard dynamic variable array vector collection matching
            match expressions::resolve_value(source_expr, scope_stack) {
                Value::Array(arr) => arr,
                _ => Vec::new(),
            }
        };

        let mut rendered_block = String::new();
        let inner_raw_html = tag.inner_html(parser).to_string();

        // 4. Iterate over rows and apply contextual scoped variables
        for (idx, loop_item) in iterations.into_iter().enumerate() {
            let mut iteration_scope = json!({
                item_var_name: loop_item
            });

            // If user requested an explicit counter tracker, inject it into the local state map
            if let Some(index_name) = index_var_name {
                iteration_scope.as_object_mut().unwrap().insert(
                    index_name.to_string(),
                    json!(idx)
                );
            }

            scope_stack.push(iteration_scope);
            let compiled_child = crate::compile_dom_tree(&inner_raw_html, scope_stack);
            rendered_block.push_str(&compiled_child);
            scope_stack.pop();
        }

        ProcessingResult::OverrideInner(rendered_block)
    }
}

// =========================================================================
// ISOLATED FOR-LOOP PIPELINE TESTS
// =========================================================================
#[cfg(test)]
mod tests {
    use crate::compile_html;

    #[test]
    fn test_loop_iteration_legacy_fallback() {
        // FIX: Wrap the array inside an object with an "items" key
        let template = r#"<div x-data="{ 'items': [{'name':'A'}, {'name':'B'}] }"><ul x-for="items"><li x-text="item.name">Tmp</li></ul></div>"#;
        assert_eq!(compile_html(template), "<div><ul><li>A</li><li>B</li></ul></div>");
    }

    #[test]
    fn test_loop_iteration_standard_alpine_syntax() {
        // FIX: Wrap the array inside an object with an "items" key
        let template = r#"<div x-data="{ 'items': [{'name':'X'}, {'name':'Y'}] }"><ul x-for="element in items"><li x-text="element.name"></li></ul></div>"#;
        assert_eq!(compile_html(template), "<div><ul><li>X</li><li>Y</li></ul></div>");
    }

    #[test]
    fn test_loop_iteration_with_index_access() {
        // FIX: Wrap the array inside an object with an "items" key
        let template = r#"<div x-data="{ 'items': ['Red', 'Blue'] }"><ul x-for="(color, idx) in items"><li :id="`item-${idx}`" x-text="color"></li></ul></div>"#;
        assert_eq!(compile_html(template), "<div><ul><li id='item-0'>Red</li><li id='item-1'>Blue</li></ul></div>");
    }

    #[test]
    fn test_loop_iteration_over_numeric_range() {
        let template = r#"<div x-data="{}"><ul><template x-for="num in 3"><li x-text="num"></li></template></ul></div>"#;
        assert_eq!(compile_html(template), "<div><ul><template><li>1</li><li>2</li><li>3</li></template></ul></div>");
    }
}
