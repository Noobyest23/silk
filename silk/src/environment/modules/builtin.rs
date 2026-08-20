use std::collections::HashMap;

use crate::environment::{value::SilkValue, vm::VirtualMachine};

// @export Modules/Global
/*
    The global module is automatically imported into every Silk program. It provides a set of built-in functions and constants that are available globally, without the need for explicit imports.
*/
fn dereference_value(vm: &VirtualMachine, value: SilkValue) -> SilkValue {
    match value {
        SilkValue::Pointer(ptr) => vm
            .heap
            .get(&ptr)
            .cloned()
            .map(|inner| dereference_value(vm, inner))
            .unwrap_or(SilkValue::Pointer(ptr)),
        other => other,
    }
}

// @export Modules/Global#range
/*
    <h3>Signature</h3>
    <code>range(stop: Int) -> List</code><br>
    <code>range(start: Int, stop: Int, step: Int = 1) -> List</code>

    <p>Returns an array of integers within a specified interval.</p>

    <b>Parameters:</b>
    - <code>start</code>: The starting integer of the sequence (inclusive). Defaults to <code>0</code> if only one argument is provided.
    - <code>stop</code>: The boundary integer of the sequence (exclusive).
    - <code>step</code>: The increment value between each integer. Can be negative for descending ranges. Defaults to <code>1</code>.

    <b>Returns:</b>
    - <code>List</code>: A list containing the generated integer sequence.

    <b>Usage:</b>
    <pre><code>range(0, 10)    // => [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
range(5)        // => [0, 1, 2, 3, 4]
range(1, 10, 2) // => [1, 3, 5, 7, 9]
range(10, 0, -1) // => [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]</code></pre>
*/
pub fn silk_builtin_range(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    let (start, stop, step) = match args.len() {
        1 => (0, args[0].as_int().unwrap_or(0), 1),
        2 => (args[0].as_int().unwrap_or(0), args[1].as_int().unwrap_or(0), 1),
        3 => (
            args[0].as_int().unwrap_or(0),
            args[1].as_int().unwrap_or(0),
            args[2].as_int().unwrap_or(1),
        ),
        _ => {
            vm.error("range() expects 1 to 3 integer arguments".to_string());
            return SilkValue::Null;
        }
    };

    if step == 0 {
        vm.error("range() step argument must not be zero".to_string());
        return SilkValue::Null;
    }

    let mut elements = Vec::new();
    let mut current = start;

    if step > 0 {
        while current < stop {
            elements.push(SilkValue::Int(current));
            current += step;
        }
    } else {
        while current > stop {
            elements.push(SilkValue::Int(current));
            current += step;
        }
    }

    let ptr = vm.next_heap_ptr;
    vm.heap_allocate(SilkValue::List(elements));
    SilkValue::Pointer(ptr)
}

// @export Modules/Global#Int
/*
    <b>Signature</b>
    <code>Int(value: Any) -> Int</code>

    <p>Integer Constructor. Converts a given value to an integer. Handles integers, floats, numeric strings, and booleans.</p>

    <b>Parameters:</b>
    - <code>value</code>: The value to convert into an integer.

    <b>Returns:</b>
    - <code>Int</code>: The converted integer value.

    <b>Usage:</b>
    <pre><code>Int("42") // => 42
Int(3.14) // => 3
Int(true) // => 1</code></pre>
*/
pub fn silk_builtin_int(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error("Int() expects exactly 1 argument".to_string());
        return SilkValue::Null;
    }

    let value = dereference_value(vm, args[0].clone());

    match value {
        SilkValue::Int(i) => SilkValue::Int(i),
        SilkValue::Float(f) => SilkValue::Int(f as i32),
        SilkValue::String(s) => match s.parse::<i32>() {
            Ok(value) => SilkValue::Int(value),
            Err(_) => {
                vm.error(format!("Int() could not parse '{}' as an integer", s));
                SilkValue::Null
            }
        },
        other => {
            vm.error(format!("Int() cannot convert value of type {}", other));
            SilkValue::Null
        }
    }
}

// @export Modules/Global#String
/*
    <h3>Signature</h3>
    <code>String(value: Any) -> String</code>

    <p>String Constructor. Converts a given value to its string representation. Supports integers, floats, booleans, strings, and null.</p>

    <b>Parameters:</b>
    - <code>value</code>: The value to convert into a string.

    <b>Returns:</b>
    - <code>String</code>: The converted string.

    <b>Usage:</b>
    <pre><code>String(100)  // => "100"
String(true) // => "true"
String(null) // => "null"</code></pre>
*/
pub fn silk_builtin_string(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error("String() expects exactly 1 argument".to_string());
        return SilkValue::Null;
    }

    let value = dereference_value(vm, args[0].clone());

    let value = match value {
        SilkValue::String(s) => s,
        SilkValue::Int(i) => i.to_string(),
        SilkValue::Float(f) => f.to_string(),
        SilkValue::Bool(b) => b.to_string(),
        SilkValue::Null => "null".to_string(),
        other => {
            vm.error(format!("String() cannot convert value of type {}", other));
            return SilkValue::Null;
        }
    };

    let ptr = vm.next_heap_ptr;
    vm.heap_allocate(SilkValue::String(value));
    SilkValue::Pointer(ptr)
}

// @export Modules/Global#Float
/*
    <h3>Signature</h3>
    <code>Float(value: Any) -> Float</code>

    <p>Float Constructor. Converts a value to a floating-point number. Accepts integers, floats, strings, booleans, and null.</p>

    <b>Parameters:</b>
    - <code>value</code>: The value to convert into a float.

    <b>Returns:</b>
    - <code>Float</code>: The converted floating-point number.

    <b>Usage:</b>
    <pre><code>Float("3.14") // => 3.14
Float(10)     // => 10.0
Float(true)   // => 1.0</code></pre>
*/
pub fn silk_builtin_float(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error("Float() expects exactly 1 argument".to_string());
        return SilkValue::Null;
    }

    let value = dereference_value(vm, args[0].clone());

    match value {
        SilkValue::Float(f) => SilkValue::Float(f),
        SilkValue::Int(i) => SilkValue::Float(i as f32),
        SilkValue::Bool(b) => SilkValue::Float(if b { 1.0 } else { 0.0 }),
        SilkValue::String(s) => match s.parse::<f32>() {
            Ok(value) => SilkValue::Float(value),
            Err(_) => {
                vm.error(format!("Float() could not parse '{}' as a float", s));
                SilkValue::Null
            }
        },
        SilkValue::Null => SilkValue::Float(0.0),
        other => {
            vm.error(format!("Float() cannot convert value of type {}", other));
            SilkValue::Null
        }
    }
}

// @export Modules/Global#Bool
/*
    <h3>Signature</h3>
    <code>Bool(value: Any) -> Bool</code>

    <p>Bool Constructor. Converts a value to a boolean. Evaluates common truthy/falsy strings ("true", "false", "1", "0", "yes", "no"), numbers, null, and object truthiness.</p>

    <b>Parameters:</b>
    - <code>value</code>: The value to convert into a boolean.

    <b>Returns:</b>
    - <code>Bool</code>: The evaluated boolean result.

    <b>Usage:</b>
    <pre><code>Bool("yes") // => true
Bool(0)     // => false
Bool(null)  // => false</code></pre>
*/
pub fn silk_builtin_bool(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error("Bool() expects exactly 1 argument".to_string());
        return SilkValue::Null;
    }

    let value = dereference_value(vm, args[0].clone());

    match value {
        SilkValue::Bool(b) => SilkValue::Bool(b),
        SilkValue::Int(i) => SilkValue::Bool(i != 0),
        SilkValue::Float(f) => SilkValue::Bool(f != 0.0),
        SilkValue::String(s) => {
            let lowered = s.trim().to_lowercase();
            match lowered.as_str() {
                "true" | "1" | "yes" | "y" | "on" => SilkValue::Bool(true),
                "false" | "0" | "no" | "n" | "off" => SilkValue::Bool(false),
                _ => {
                    vm.error(format!("Bool() could not parse '{}' as a boolean", s));
                    SilkValue::Null
                }
            }
        }
        SilkValue::Null => SilkValue::Bool(false),
        other => SilkValue::Bool(other.is_truthy()),
    }
}

// @export Modules/Global#List
/*
    <h3>Signature</h3>
    <code>List(*values: Any) -> List</code>

    <p>List Constructor. Creates a list from provided values. If a single list argument is passed, returns that list. If multiple values are provided, wraps them into a new list. If no arguments are passed, returns an empty list.</p>

    <b>Parameters:</b>
    - <code>*values</code>: Zero or more elements to populate the list.

    <b>Returns:</b>
    - <code>List</code>: The resulting list.

    <b>Usage:</b>
    <pre><code>List()          // => []
List(1, 2, 3)   // => [1, 2, 3]
List([4, 5])    // => [4, 5]</code></pre>
*/
pub fn silk_builtin_list(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    let list = if args.is_empty() {
        Vec::new()
    } else if args.len() == 1 {
        let value = dereference_value(vm, args[0].clone());
        match value {
            SilkValue::List(items) => items,
            other => vec![other],
        }
    } else {
        args.iter().cloned().map(|arg| dereference_value(vm, arg)).collect()
    };

    let ptr = vm.next_heap_ptr;
    vm.heap_allocate(SilkValue::List(list));
    SilkValue::Pointer(ptr)
}

pub fn build_builtin_map() -> HashMap<String, SilkValue> {
    let mut map = HashMap::new();
    map.insert(
        "range".to_string(),
        SilkValue::NativeFn(
            silk_builtin_range,
            String::from("range(start_or_stop: Int, stop: Int = null, step: Int = 1) -> Array; Generates an array of integers within a specified interval."),
        ),
    );
    map.insert(
        "Int".to_string(),
        SilkValue::NativeFn(
            silk_builtin_int,
            String::from("Int(value: Any) -> Int; Converts a value to an integer."),
        ),
    );
    map.insert(
        "Float".to_string(),
        SilkValue::NativeFn(
            silk_builtin_float,
            String::from("Float(value: Any) -> Float; Converts a value to a float."),
        ),
    );
    map.insert(
        "Bool".to_string(),
        SilkValue::NativeFn(
            silk_builtin_bool,
            String::from("Bool(value: Any) -> Bool; Converts a value to a boolean."),
        ),
    );
    map.insert(
        "String".to_string(),
        SilkValue::NativeFn(
            silk_builtin_string,
            String::from("String(value: Any) -> String; Converts a value to a string."),
        ),
    );
    map.insert(
        "List".to_string(),
        SilkValue::NativeFn(
            silk_builtin_list,
            String::from("List(value1: Any, value2: Any, ...) -> List; Creates a list from the provided values."),
        ),
    );
    map
}