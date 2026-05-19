use axum::{response::Html, routing::get, Router};
use serde_json::Value;
use std::fs;
use tokio::net::TcpListener;

mod attributes;
use attributes::{AttributeProcessor, ProcessingResult, ForLoopProcessor, TextProcessor, ConditionalProcessor};

/// Recursively processes HTML elements against a scope environment stack
pub fn compile_dom_tree(html: &str, scope_stack: &mut Vec<Value>) -> String {
    let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let mut compiled_output = String::new();

    // Register processors: x-if runs first because if it's false, we drop everything early!
    let processors: Vec<Box<dyn AttributeProcessor>> = vec![
        Box::new(ConditionalProcessor),
        Box::new(ForLoopProcessor),
        Box::new(TextProcessor),
    ];

    for child_node_handle in dom.children() {
        if let Some(node) = child_node_handle.get(parser) {
            match node {
                tl::Node::Tag(tag) => {
                    let tag_name = tag.name().as_utf8_str();
                    let attributes = tag.attributes();
                    let mut pushed_data_scope = false;

                    // 1. Context initialization (x-data)
                    if let Some(Some(x_data_raw)) = attributes.get("x-data") {
                        let normalized = x_data_raw.as_utf8_str().replace('\'', "\"");
                        if let Ok(parsed_json) = serde_json::from_str::<Value>(&normalized) {
                            scope_stack.push(parsed_json);
                            pushed_data_scope = true;
                        }
                    }

                    // 2. Execute Extensible Attribute Pipeline Interceptors
                    let mut inner_content_override = None;
                    let mut skip_tag = false;

                    for processor in &processors {
                        if let Some(Some(attr_raw)) = attributes.get(processor.name()) {
                            let attr_val = attr_raw.as_utf8_str();
                            
                            match processor.process(tag, &attr_val, scope_stack, parser) {
                                ProcessingResult::SkipEntireTag => {
                                    skip_tag = true;
                                    break; // Break the pipeline immediately
                                }
                                ProcessingResult::OverrideInner(custom_html) => {
                                    inner_content_override = Some(custom_html);
                                    break; 
                                }
                                ProcessingResult::Continue => {}
                            }
                        }
                    }

                    // If an interceptor (like x-if) triggered a skip, clean up scope and jump to next node
                    if skip_tag {
                        if pushed_data_scope { scope_stack.pop(); }
                        continue;
                    }

                    // 3. Serialize attributes back safely, stripping framework helper tokens
                    let mut attrs_str = String::new();
                    for (key, val) in attributes.iter() {
                        if key != "x-data" && !processors.iter().any(|p| p.name() == key) {
                            if let Some(v) = val {
                                attrs_str.push_str(&format!(" {}='{}'", key, v.as_ref()));
                            } else {
                                attrs_str.push_str(&format!(" {}", key));
                            }
                        }
                    }

                    // 4. Render Tag Contents based on Pipeline Interceptor decision
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
