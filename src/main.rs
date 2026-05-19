use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde_json::Value;
use std::{fs, path::PathBuf};
use tokio::net::TcpListener;

mod attributes;
use attributes::{
    bind, AttributeProcessor, BindProcessor, ConditionalProcessor, ForLoopProcessor,
    IncludeProcessor, ProcessingResult, TextProcessor,
};

pub fn compile_dom_tree(html: &str, scope_stack: &mut Vec<Value>) -> String {
    let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let mut compiled_output = String::new();

    let processors: Vec<Box<dyn AttributeProcessor>> = vec![
        Box::new(IncludeProcessor), // Evaluates first to unpack partials before checking conditional logic
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
                            match parsed_json {
                                // SMART WRAPPER: Automatically maps raw top-level JSON arrays 
                                // to 'items' and 'item' keys for backwards and Alpine-syntax compatibility
                                Value::Array(arr) => {
                                    let wrapped_scope = serde_json::json!({
                                        "items": arr.clone(),
                                        "item": arr.clone()
                                    });
                                    scope_stack.push(wrapped_scope);
                                }
                                // Standard objects get pushed straight through unmodified
                                Value::Object(_) => {
                                    scope_stack.push(parsed_json);
                                }
                                _ => {
                                    scope_stack.push(parsed_json);
                                }
                            }
                            pushed_data_scope = true;
                        }
                    }

                    let mut inner_content_override = None;
                    let mut skip_tag = false;

                    for processor in &processors {
                        let has_attr = attributes.iter().any(|(k, _)| {
                            let k_ref = k.as_ref();
                            k_ref == processor.name()
                                || k_ref.starts_with(&format!("{}:", processor.name()))
                                || (processor.name() == "x-bind" && k_ref.starts_with(':'))
                        });

                        if has_attr {
                            let attr_val = attributes
                                .iter()
                                .find(|(k, _)| {
                                    let k_ref = k.as_ref();
                                    k_ref.starts_with(processor.name())
                                        || (processor.name() == "x-bind" && k_ref.starts_with(':'))
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
                        if pushed_data_scope {
                            scope_stack.pop();
                        }
                        continue;
                    }

                    let attrs_str = bind::evaluate_and_bind_attributes(
                        tag,
                        scope_stack,
                        &processor_names_raw,
                    );

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

                    compiled_output.push_str(&format!(
                        "<{}{}>{}</{}>",
                        tag_name, attrs_str, final_inner_content, tag_name
                    ));

                    if pushed_data_scope {
                        scope_stack.pop();
                    }
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

pub fn compile_html(raw_html: &str) -> String {
    let mut scope_stack = vec![];
    compile_dom_tree(raw_html, &mut scope_stack)
}

/// Dynamic Router handling static binary pass-throughs or template interpretation on the fly
async fn handle_templates(Path(requested_path): Path<String>) -> impl IntoResponse {
    let safe_path = requested_path.trim_start_matches('/');
    if safe_path.contains("..") {
        return (StatusCode::BAD_REQUEST, "400: Bad Request").into_response();
    }

    let mut target_file = PathBuf::from("public").join(safe_path);

    if target_file.is_dir() || safe_path.is_empty() {
        target_file = target_file.join("index.html");
    } else if target_file.extension().is_none() {
        target_file.set_extension("html");
    }

    match fs::read(target_file.clone()) {
        Ok(bytes) => {
            let extension = target_file
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            let mut headers = HeaderMap::new();

            match extension {
                "html" => {
                    headers.insert(
                        header::CONTENT_TYPE,
                        "text/html; charset=utf-8".parse().unwrap(),
                    );
                    let raw_html = String::from_utf8_lossy(&bytes);
                    let processed_html = compile_html(&raw_html);
                    (StatusCode::OK, headers, processed_html).into_response()
                }
                "txt" => {
                    headers.insert(
                        header::CONTENT_TYPE,
                        "text/plain; charset=utf-8".parse().unwrap(),
                    );
                    (StatusCode::OK, headers, bytes).into_response()
                }
                "css" => {
                    headers.insert(header::CONTENT_TYPE, "text/css".parse().unwrap());
                    (StatusCode::OK, headers, bytes).into_response()
                }
                "js" => {
                    headers.insert(
                        header::CONTENT_TYPE,
                        "application/javascript".parse().unwrap(),
                    );
                    (StatusCode::OK, headers, bytes).into_response()
                }
                "png" => {
                    headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());
                    (StatusCode::OK, headers, bytes).into_response()
                }
                "jpg" | "jpeg" => {
                    headers.insert(header::CONTENT_TYPE, "image/jpeg".parse().unwrap());
                    (StatusCode::OK, headers, bytes).into_response()
                }
                "svg" => {
                    headers.insert(header::CONTENT_TYPE, "image/svg+xml".parse().unwrap());
                    (StatusCode::OK, headers, bytes).into_response()
                }
                _ => {
                    headers.insert(
                        header::CONTENT_TYPE,
                        "application/octet-stream".parse().unwrap(),
                    );
                    (StatusCode::OK, headers, bytes).into_response()
                }
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            format!(
                "<h1>404: File Not Found</h1><p>Looked for: {:?}</p>",
                target_file
            ),
        )
            .into_response(),
    }
}

async fn handle_root() -> impl IntoResponse {
    handle_templates(Path(String::new())).await
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handle_root))
        .route("/*path", get(handle_templates));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🚀 Project X Server-Side Template Matrix active on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
