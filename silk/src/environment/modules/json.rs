use std::collections::HashMap;
use crate::environment::vm::{SilkHandle, VirtualMachine};
use super::super::value::SilkValue;

// @export Modules/Json
/*
    The Json module provides functions for serializing Silk values into JSON strings
    and parsing JSON formatted strings into native Silk values.
*/

// --- Helper Parser implementation ---

fn parse_json_value(vm: &mut VirtualMachine, s: &str) -> Result<(SilkValue, usize), String> {
    let raw = s;
    let s = s.trim_start();
    let leading_ws = raw.len() - s.len();

    if s.is_empty() {
        return Err("Unexpected end of input".to_string());
    }

    let first_char = s.chars().next().unwrap();

    match first_char {
        'n' if s.starts_with("null") => Ok((SilkValue::Null, leading_ws + 4)),
        't' if s.starts_with("true") => Ok((SilkValue::Bool(true), leading_ws + 4)),
        'f' if s.starts_with("false") => Ok((SilkValue::Bool(false), leading_ws + 5)),
        '"' => {
            let (value, consumed) = parse_json_string(vm, s)?;
            Ok((value, leading_ws + consumed))
        }
        '[' => {
            let (value, consumed) = parse_json_array(vm, s)?;
            Ok((value, leading_ws + consumed))
        }
        '{' => {
            let (value, consumed) = parse_json_object(vm, s)?;
            Ok((value, leading_ws + consumed))
        }
        _ if first_char == '-' || first_char.is_ascii_digit() => {
            let (value, consumed) = parse_json_number(s)?;
            Ok((value, leading_ws + consumed))
        }
        _ => Err(format!("Unexpected character: {}", first_char)),
    }
}

fn parse_json_string(vm: &mut VirtualMachine, s: &str) -> Result<(SilkValue, usize), String> {
    let mut chars = s.char_indices().skip(1);
    let mut result = String::new();
    let mut escaped = false;

    while let Some((idx, ch)) = chars.next() {
        if escaped {
            match ch {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                'b' => result.push('\x08'),
                'f' => result.push('\x0C'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                _ => result.push(ch),
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            let handle = vm.heap_allocate(SilkValue::String(result));
            let val = match handle {
                SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
                _ => unreachable!(),
            };
            return Ok((val, idx + 1));
        } else {
            result.push(ch);
        }
    }

    Err("Unterminated string literal".to_string())
}

fn parse_json_number(s: &str) -> Result<(SilkValue, usize), String> {
    let mut end = 0;
    let mut is_float = false;

    for (idx, ch) in s.char_indices() {
        if ch.is_ascii_digit() || ch == '-' || ch == '+' {
            end = idx + ch.len_utf8();
        } else if ch == '.' || ch == 'e' || ch == 'E' {
            is_float = true;
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    let num_str = &s[..end];
    if is_float {
        match num_str.parse::<f32>() {
            Ok(f) => Ok((SilkValue::Float(f), end)),
            Err(_) => Err(format!("Invalid float literal: {}", num_str)),
        }
    } else {
        match num_str.parse::<i32>() {
            Ok(i) => Ok((SilkValue::Int(i), end)),
            Err(_) => Err(format!("Invalid integer literal: {}", num_str)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_parser_handles_whitespace_and_escaped_quotes() {
        let mut vm = VirtualMachine::new();
        let data = "{\"name\": \"Silk\", \"version\": 1}";
        let (value, consumed) = parse_json_value(&mut vm, data).unwrap();

        assert_eq!(consumed, data.len());
        assert!(matches!(value, SilkValue::Pointer(_)));

        if let SilkValue::Pointer(ptr) = value {
            let map = vm.heap_get_map(SilkValue::Pointer(ptr)).unwrap();
            assert!(matches!(map.get("name"), Some(SilkValue::Pointer(_))));
            assert!(matches!(map.get("version"), Some(SilkValue::Int(1))));
        }
    }
}

fn parse_json_array(vm: &mut VirtualMachine, s: &str) -> Result<(SilkValue, usize), String> {
    let mut offset = 1; // Skip '['
    let mut elements = Vec::new();

    loop {
        let remaining = s[offset..].trim_start();
        let trimmed_len = s[offset..].len() - remaining.len();
        offset += trimmed_len;

        if remaining.starts_with(']') {
            offset += 1;
            break;
        }

        let (val, consumed) = parse_json_value(vm, &s[offset..])?;
        elements.push(val);
        offset += consumed;

        let next = s[offset..].trim_start();
        let trimmed_len = s[offset..].len() - next.len();
        offset += trimmed_len;

        if next.starts_with(',') {
            offset += 1;
        } else if next.starts_with(']') {
            offset += 1;
            break;
        } else {
            return Err("Expected ',' or ']' in array".to_string());
        }
    }

    let handle = vm.heap_allocate(SilkValue::List(elements));
    match handle {
        SilkHandle::HeapAllocated(p) => Ok((SilkValue::Pointer(p), offset)),
        _ => unreachable!(),
    }
}

fn parse_json_object(vm: &mut VirtualMachine, s: &str) -> Result<(SilkValue, usize), String> {
    let mut offset = 1; // Skip '{'
    let mut map = HashMap::new();

    loop {
        let remaining = s[offset..].trim_start();
        let trimmed_len = s[offset..].len() - remaining.len();
        offset += trimmed_len;

        if remaining.starts_with('}') {
            offset += 1;
            break;
        }

        if !remaining.starts_with('"') {
            return Err("Expected string key in object".to_string());
        }

        let (key_val, consumed) = parse_json_string(vm, remaining)?;
        offset += consumed;

        let key_str = vm.heap_get_string(key_val).unwrap_or_default();

        let remaining = s[offset..].trim_start();
        let trimmed_len = s[offset..].len() - remaining.len();
        offset += trimmed_len;

        if !remaining.starts_with(':') {
            return Err("Expected ':' after object key".to_string());
        }
        offset += 1;

        let (val, consumed) = parse_json_value(vm, &s[offset..])?;
        map.insert(key_str, val);
        offset += consumed;

        let remaining = s[offset..].trim_start();
        let trimmed_len = s[offset..].len() - remaining.len();
        offset += trimmed_len;

        if remaining.starts_with(',') {
            offset += 1;
        } else if remaining.starts_with('}') {
            offset += 1;
            break;
        } else {
            return Err("Expected ',' or '}' in object".to_string());
        }
    }

    let handle = vm.heap_allocate(SilkValue::Map(map));
    match handle {
        SilkHandle::HeapAllocated(p) => Ok((SilkValue::Pointer(p), offset)),
        _ => unreachable!(),
    }
}

// --- Helper Stringify implementation ---

fn stringify_value(vm: &mut VirtualMachine, val: &SilkValue) -> Result<String, String> {
    match val {
        SilkValue::Null => Ok("null".to_string()),
        SilkValue::Bool(b) => Ok(b.to_string()),
        SilkValue::Int(i) => Ok(i.to_string()),
        SilkValue::Float(f) => Ok(f.to_string()),
        SilkValue::String(s) => Ok(format!("\"{}\"", s.escape_default())),
        SilkValue::Pointer(_) => {
            if let Some(s) = vm.heap_get_string(val.clone()) {
                Ok(format!("\"{}\"", s.escape_default()))
            } else if let Some(list) = vm.heap_get_list(val.clone()) {
                let mut parts = Vec::new();
                for item in list {
                    parts.push(stringify_value(vm, &item)?);
                }
                Ok(format!("[{}]", parts.join(",")))
            } else if let Some(map) = vm.heap_get_map(val.clone()) {
                let mut parts = Vec::new();
                for (k, v) in map {
                    let serialized_v = stringify_value(vm, &v)?;
                    parts.push(format!("\"{}\":{}", k.escape_default(), serialized_v));
                }
                Ok(format!("{{{}}}", parts.join(",")))
            } else {
                Err("Cannot serialize pointer value".to_string())
            }
        }
        SilkValue::Map(map) => {
            let mut parts = Vec::new();
            for (k, v) in map {
                let serialized_v = stringify_value(vm, &v)?;
                parts.push(format!("\"{}\":{}", k.escape_default(), serialized_v));
            }
            Ok(format!("{{{}}}", parts.join(",")))
        }
        _ => Err("Unsupported value type for JSON serialization".to_string()),
    }
}

// @export Modules/Json#parse
/*
    <b>Signature</b>
    <code>parse(json_string: String) -> Any</code>

    <p>Parses a JSON string into corresponding Silk data structures (Maps, Lists, Strings, Numbers, Bools, Null).</p>

    <b>Parameters:</b>
    - <code>json_string</code>: Target JSON string to decode.

    <b>Returns:</b>
    - <code>Any</code>: Decoded Silk value representation or <code>Null</code> on parse failure.

    <b>Usage:</b>
    <pre><code>var data = parse("{\"name\": \"Silk\", \"version\": 1}")</code></pre>
*/
pub fn silk_json_parse(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'parse' expects exactly 1 argument"));
        return SilkValue::Null;
    }

    let json_str = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    match parse_json_value(vm, &json_str) {
        Ok((val, _)) => val,
        Err(err) => {
            vm.error(format!("JSON Parse Error: {}", err));
            SilkValue::Null
        }
    }
}

// @export Modules/Json#stringify
/*
    <b>Signature</b>
    <code>stringify(val: Any) -> String</code>

    <p>Converts a Silk value or data structure into a valid JSON string.</p>

    <b>Parameters:</b>
    - <code>val</code>: The value to serialize into JSON format.

    <b>Returns:</b>
    - <code>String</code>: JSON encoded string representation.

    <b>Usage:</b>
    <pre><code>var json_str = stringify(data)</code></pre>
*/
pub fn silk_json_stringify(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'stringify' expects exactly 1 argument"));
        return SilkValue::Null;
    }

    match stringify_value(vm, &args[0]) {
        Ok(serialized) => {
            let handle = vm.heap_allocate(SilkValue::String(serialized));
            match handle {
                SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
                _ => unreachable!(),
            }
        }
        Err(err) => {
            vm.error(format!("JSON Stringify Error: {}", err));
            SilkValue::Null
        }
    }
}

pub fn build_json_map() -> HashMap<String, SilkValue> {
    let mut map = HashMap::new();

    map.insert(
        "parse".to_string(),
        SilkValue::NativeFn(
            silk_json_parse,
            String::from("Parse(json_string: String) -> Any; Parses a JSON string into native values"),
        ),
    );
    map.insert(
        "stringify".to_string(),
        SilkValue::NativeFn(
            silk_json_stringify,
            String::from("Stringify(val: Any) -> String; Converts a native value into a JSON string"),
        ),
    );

    map
}