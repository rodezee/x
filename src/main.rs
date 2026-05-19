use axum::{response::Html, routing::get, Router};
use serde_json::Value;
use std::fs;
use tokio::net::TcpListener;

mod attributes;
use attributes::{AttributeProcessor, ProcessingResult, ForLoopProcessor, TextProcessor, ConditionalProcessor, BindProcessor, bind};

pub fn compile_dom_tree(html: &str, scope_stack: &mut Vec<Value>) -> String {
    let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let mut compiled_output = String::new();

    let processors: Vec<Box<dyn AttributeProcessor>> = vec![
        Box::new(ConditionalProcessor),
        Box::new(ForLoopProcessor),
        Box::new(TextProcessor),
        Box::new(BindProcessor),
    ];

    let processor_names_raw: Vec<&str> = processors.iter().map(|p| p.name()).collect();

    for child_node_handle in dom.children() {
        if let Some(node) = child_node_handle.get(parser) {
            match node {
                tl::Node::Tag(tag) => {
                    let tag_name = tag.name().as_utf8_str();
                    let attributes = tag.attributes();
                    let mut pushed_data_scope = false;

                    if let Some(Some(x_data_raw)) = attributes.get("x-data") {
                        let normalized = x_data_raw.as_utf8_str().replace('\'', "\"");
                        if let Ok(parsed_json) = serde_json::from_str::<Value>(&normalized) {
                            scope_stack.push(parsed_json);
                            pushed_data_scope = true;
                        }
                    }

                    let mut inner_content_override = None;
                    let mut skip_tag = false;

                    for processor in &processors {
                        // FIX: Use k.as_ref() directly here
                        let has_attr = attributes.iter().any(|(k, _)| {
                            let k_ref = k.as_ref();
                            k_ref == processor.name() || 
                            k_ref.starts_with(&format!("{}:", processor.name())) ||
                            (processor.name() == "x-bind" && k_ref.starts_with(':'))
                        });

                        if has_attr {
                            // FIX: Use k.as_ref() directly here too
                            let attr_val = attributes.iter()
                                .find(|(k, _)| {
                                    let k_ref = k.as_ref();
                                    k_ref.starts_with(processor.name()) || (processor.name() == "x-bind" && k_ref.starts_with(':'))
                                })
                                .and_then(|(_, v)| v.as_ref().map(|s| s.as_ref().to_string()))
                                .unwrap_or_default();

                            match processor.process(tag, &attr_val, scope_stack, parser) {
                                ProcessingResult::SkipEntireTag => {
                                    skip_tag = true;
                                    break;
                                }
                                ProcessingResult::OverrideInner(custom_html) => {
                                    inner_content_override = Some(custom_html);
                                    break; 
                                }
                                ProcessingResult::Continue => {}
                            }
                        }
                    }

                    if skip_tag {
                        if pushed_data_scope { scope_stack.pop(); }
                        continue;
                    }

                    let attrs_str = bind::evaluate_and_bind_attributes(tag, scope_stack, &processor_names_raw);

                    let final_inner_content = match inner_content_override {
                        Some(custom_html) => custom_html,
                        None => {
                            let inner_raw_html = tag.inner_html(parser).to_string();
                            if !inner_raw_html.is_empty() {
                                compile_dom_tree(&inner_raw_html, scope_stack)
                            } else {
                                String::new()
                            }
                        }
                    };

                    compiled_output.push_str(&format!("<{}{}>{}</{}>", tag_name, attrs_str, final_inner_content, tag_name));

                    if pushed_data_scope { scope_stack.pop(); }
                }
                tl::Node::Raw(raw_text) => {
                    compiled_output.push_str(&raw_text.as_utf8_str());
                }
                _ => {}
            }
        }
    }
    compiled_output
}

fn compile_html(raw_html: &str) -> String {
    let mut scope_stack = vec![];
    compile_dom_tree(raw_html, &mut scope_stack)
}

async fn handle_index() -> Html<String> {
    let raw_html = fs::read_to_string("index.html")
        .unwrap_or_else(|_| "<h1>500: index.html missing</h1>".to_string());
    
    let processed_html = compile_html(&raw_html);
    Html(processed_html)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handle_index));
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🚀 Project X Scope Engine running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
