use serde_json::Value;

/// Evaluates a binary condition string (e.g., "item.id === 202") against the active environment stack
pub fn evaluate_expression(expr: &str, scope_stack: &[Value]) -> bool {
    let expr = expr.trim();

    // Support simple logical inversion prefix (e.g., "!item.premium")
    if expr.starts_with('!') {
        let sub_expr = &expr[1..];
        return !evaluate_truthiness(sub_expr, scope_stack);
    }

    // Order operators by length to prevent partial matching (e.g., matching '=' inside '===')
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

    // If no binary operator was found, fallback to checking raw truthiness of the variable
    evaluate_truthiness(expr, scope_stack)
}

/// Helper to deeply look up variables or parse raw string/number literals
fn resolve_value(token: &str, scope_stack: &[Value]) -> Value {
    // 1. Check if token is a string literal wrapped in single quotes
    if token.starts_with('\'') && token.ends_with('\'') {
        return Value::String(token[1..token.len() - 1].to_string());
    }

    // 2. Check if token is a numeric literal
    if let Ok(num) = token.parse::<i64>() {
        return Value::from(num);
    }
    if let Ok(num) = token.parse::<f64>() {
        return Value::from(num);
    }

    // 3. Check for keywords
    if token == "true" { return Value::Bool(true); }
    if token == "false" { return Value::Bool(false); }
    if token == "null" { return Value::Null; }

    // 4. Otherwise, handle variable resolution across stack contexts
    let parts: Vec<&str> = token.split('.').collect();
    let (namespace, field_key) = if parts.len() == 2 {
        (Some(parts[0]), parts[1])
    } else {
        (None, parts[0])
    };

    for scope in scope_stack.iter().rev() {
        let value = if let Some(ns) = namespace {
            if let Some(Value::Object(map)) = scope.get(ns) {
                map.get(field_key)
            } else if scope.is_object() && scope.get(field_key).is_some() && ns == "item" {
                scope.get(field_key)
            } else {
                None
            }
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
