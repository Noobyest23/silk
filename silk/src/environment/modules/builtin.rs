use std::collections::HashMap;

use crate::environment::{value::SilkValue, vm::VirtualMachine};

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
