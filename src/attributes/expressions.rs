use serde_json::Value;

/// Evaluates a binary condition string (e.g., "item.id === 202") against the active environment stack
pub fn evaluate_expression(expr: &str, scope_stack: &[Value]) -> bool {
    let expr = expr.trim();

    if expr.starts_with('!') {
        let sub_expr = &expr[1..];
        return !evaluate_truthiness(sub_expr, scope_stack);
    }

    let operators = ["===", "!==", "==", "!=", "<=", ">=", "<", ">"];
    
    for op in operators.iter() {
        if let Some(index) = expr.find(op) {
            let left_raw = expr[..index].trim();
            let right_raw = expr[index + op.len()..].trim();

            let left_val = resolve_value(left_raw, scope_stack);
            let right_val = resolve_value(right_raw, scope_stack);

            return match *op {
                "===" | "==" => left_val == right_val,
                "!==" | "!=" => left_val != right_val,
                "<" => compare_numbers(&left_val, &right_val, |a, b| a < b),
                ">" => compare_numbers(&left_val, &right_val, |a, b| a > b),
                "<=" => compare_numbers(&left_val, &right_val, |a, b| a <= b),
                ">=" => compare_numbers(&left_val, &right_val, |a, b| a >= b),
                _ => false,
            };
        }
    }

    evaluate_truthiness(expr, scope_stack)
}

/// NEW: Evaluates complex mixed strings (e.g., "`color: ${item.hex}; font-size: 14px;`")
pub fn evaluate_complex_string(expr: &str, scope_stack: &[Value]) -> String {
    let expr = expr.trim();

    // Check for JavaScript-style backtick template literal formatting: `text ${var}`
    if expr.starts_with('`') && expr.ends_with('`') {
        let mut result = String::new();
        let content = &expr[1..expr.len() - 1];
        let mut current_pos = 0;

        while let Some(start_idx) = content[current_pos..].find("${") {
            let absolute_start = current_pos + start_idx;
            // Push literal text leading up to the token variable
            result.push_str(&content[current_pos..absolute_start]);

            if let Some(end_idx) = content[absolute_start..].find('}') {
                let absolute_end = absolute_start + end_idx;
                let var_token = content[absolute_start + 2..absolute_end].trim();
                
                // Resolve the token variable from our environment stack
                let resolved = resolve_value(var_token, scope_stack);
                let resolved_str = match resolved {
                    Value::String(s) => s,
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => String::new(),
                };
                
                result.push_str(&resolved_str);
                current_pos = absolute_end + 1;
            } else {
                break;
            }
        }
        
        // Push any remaining trailing text
        result.push_str(&content[current_pos..]);
        return result;
    }

    // Fallback to checking normal variable lookup
    let resolved = resolve_value(expr, scope_stack);
    match resolved {
        Value::String(s) => s,
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "undefined".to_string(),
        _ => resolved.to_string(),
    }
}

pub fn resolve_value(token: &str, scope_stack: &[Value]) -> Value {
    let token = token.trim();
    if token.starts_with('\'') && token.ends_with('\'') {
        return Value::String(token[1..token.len() - 1].to_string());
    }

    if let Ok(num) = token.parse::<i64>() { return Value::from(num); }
    if let Ok(num) = token.parse::<f64>() { return Value::from(num); }
    if token == "true" { return Value::Bool(true); }
    if token == "false" { return Value::Bool(false); }
    if token == "null" { return Value::Null; }

    let parts: Vec<&str> = token.split('.').collect();
    let (namespace, field_key) = if parts.len() == 2 {
        (Some(parts[0]), parts[1])
    } else {
        (None, parts[0])
    };

    for scope in scope_stack.iter().rev() {
        let value = if let Some(ns) = namespace {
            if let Some(Value::Object(map)) = scope.get(ns) { map.get(field_key) }
            else if scope.is_object() && scope.get(field_key).is_some() && ns == "item" { scope.get(field_key) }
            else { None }
        } else {
            scope.get(field_key)
        };

        if let Some(val) = value {
            return val.clone();
        }
    }

    Value::Null
}

fn evaluate_truthiness(expression: &str, scope_stack: &[Value]) -> bool {
    match resolve_value(expression, scope_stack) {
        Value::Bool(b) => b,
        Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        Value::String(s) => !s.is_empty() && s != "false",
        Value::Array(arr) => !arr.is_empty(),
        Value::Object(_) => true,
        Value::Null => false,
    }
}

fn compare_numbers<F>(left: &Value, right: &Value, compare_op: F) -> bool 
where
    F: Fn(f64, f64) -> bool,
{
    if let (Some(l), Some(r)) = (left.as_f64(), right.as_f64()) {
        compare_op(l, r)
    } else {
        false
    }
}
