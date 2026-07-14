use std::collections::HashMap;
use crate::environment::vm::{SilkHandle, VirtualMachine};
use super::super::value::SilkValue;



pub fn silk_string_len(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        eprintln!("[Silk Error] 'len' expects exactly 1 argument");
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    
    SilkValue::Int(s.len() as i32)
}



pub fn silk_string_concat(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'concat' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let s1 = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let s2 = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    let new_string = format!("{}{}", s1, s2);
    let handle = vm.heap_allocate(SilkValue::String(new_string));
    match handle {
        SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
        _ => unreachable!(),
    }
}



pub fn silk_string_upper(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        eprintln!("[Silk Error] 'upper' expects exactly 1 argument");
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let upper_str = s.to_uppercase();
    let handle = vm.heap_allocate(SilkValue::String(upper_str));
    match handle {
        SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
        _ => unreachable!(),
    }
}



pub fn silk_string_lower(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        eprintln!("[Silk Error] 'lower' expects exactly 1 argument");
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let lower_str = s.to_lowercase();
    let handle = vm.heap_allocate(SilkValue::String(lower_str));
    match handle {
        SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
        _ => unreachable!(),
    }
}



pub fn silk_string_substring(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 3 {
        eprintln!("[Silk Error] 'substring' expects exactly 3 arguments");
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let SilkValue::Int(start) = args[1] else {
        eprintln!("[Silk Error] 'substring' argument 2 must be an integer (start index)");
        return SilkValue::Null;
    };

    let SilkValue::Int(end) = args[2] else {
        eprintln!("[Silk Error] 'substring' argument 3 must be an integer (end index)");
        return SilkValue::Null;
    };

    
    let start_idx = (start.max(0) as usize).min(s.len());
    let end_idx = (end.max(0) as usize).min(s.len());

    if start_idx > end_idx {
        eprintln!("[Silk Error] 'substring' start index cannot be greater than end index");
        return SilkValue::Null;
    }

    
    
    let sub_str: String = s.chars().skip(start_idx).take(end_idx - start_idx).collect();

    let handle = vm.heap_allocate(SilkValue::String(sub_str));
    match handle {
        SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
        _ => unreachable!(),
    }
}



pub fn silk_string_trim(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        eprintln!("[Silk Error] 'trim' expects exactly 1 argument");
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let trimmed = s.trim().to_string();
    let handle = vm.heap_allocate(SilkValue::String(trimmed));
    match handle {
        SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
        _ => unreachable!(),
    }
}



pub fn silk_string_contains(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'contains' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let haystack = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let needle = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    SilkValue::Bool(haystack.contains(&needle))
}



pub fn silk_string_replace(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 3 {
        eprintln!("[Silk Error] 'replace' expects exactly 3 arguments");
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let from_str = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    let to_str = vm.heap_get_string(args[2].clone()).unwrap_or_default();

    let replaced = s.replace(&from_str, &to_str);
    let handle = vm.heap_allocate(SilkValue::String(replaced));
    match handle {
        SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
        _ => unreachable!(),
    }
}



pub fn silk_string_reverse(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        eprintln!("[Silk Error] 'reverse' expects exactly 1 argument");
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    let reversed: String = s.chars().rev().collect();
    let handle = vm.heap_allocate(SilkValue::String(reversed));
    match handle {
        SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
        _ => unreachable!(),
    }
}



pub fn silk_string_starts_with(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'starts_with' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    let prefix = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    SilkValue::Bool(s.starts_with(&prefix))
}



pub fn silk_string_ends_with(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'ends_with' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    let suffix = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    SilkValue::Bool(s.ends_with(&suffix))
}




pub fn silk_string_index_of(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'index_of' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let haystack = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    let needle = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    match haystack.find(&needle) {
        Some(idx) => SilkValue::Int(idx as i32),
        None => SilkValue::Int(-1),
    }
}



pub fn silk_string_repeat(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'repeat' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let SilkValue::Int(count) = args[1] else {
        eprintln!("[Silk Error] 'repeat' argument 2 must be an integer (repetition count)");
        return SilkValue::Null;
    };

    if count < 0 {
        eprintln!("[Silk Error] 'repeat' count cannot be negative");
        return SilkValue::Null;
    }

    let repeated = s.repeat(count as usize);
    let handle = vm.heap_allocate(SilkValue::String(repeated));
    match handle {
        SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
        _ => unreachable!(),
    }
}




pub fn silk_string_char_at(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'char_at' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let SilkValue::Int(idx) = args[1] else {
        eprintln!("[Silk Error] 'char_at' argument 2 must be an integer (index)");
        return SilkValue::Null;
    };

    if idx < 0 {
        eprintln!("[Silk Error] 'char_at' index cannot be negative");
        return SilkValue::Null;
    }

    match s.chars().nth(idx as usize) {
        Some(ch) => {
            let char_str = ch.to_string();
            let handle = vm.heap_allocate(SilkValue::String(char_str));
            match handle {
                SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
                _ => unreachable!(),
            }
        }
        None => SilkValue::Null,
    }
}



pub fn silk_string_count(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'count' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let haystack = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    let needle = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    if needle.is_empty() {
        return SilkValue::Int(0);
    }

    let count = haystack.matches(&needle).count();
    SilkValue::Int(count as i32)
}


pub fn build_string_map() -> HashMap<String, SilkValue> {
    let mut map = HashMap::new();
    
    map.insert("len".to_string(), SilkValue::NativeFn(silk_string_len, String::from("Len(s: String) -> Int; Returns the length of a string")));
    map.insert("concat".to_string(), SilkValue::NativeFn(silk_string_concat, String::from("Concat(s1: String, s2: String) -> String; Concatenates two strings")));
    map.insert("upper".to_string(), SilkValue::NativeFn(silk_string_upper, String::from("Upper(s: String) -> String; Converts a string to uppercase")));
    map.insert("lower".to_string(), SilkValue::NativeFn(silk_string_lower, String::from("Lower(s: String) -> String; Converts a string to lowercase")));
    map.insert("substring".to_string(), SilkValue::NativeFn(silk_string_substring, String::from("Substring(s: String, start: Int, end: Int) -> String; Returns a substring of a string")));
    map.insert("trim".to_string(), SilkValue::NativeFn(silk_string_trim, String::from("Trim(s: String) -> String; Removes whitespace from the beginning and end of a string")));
    map.insert("contains".to_string(), SilkValue::NativeFn(silk_string_contains, String::from("Contains(s: String, needle: String) -> Boolean; Checks if a string contains a substring")));
    map.insert("replace".to_string(), SilkValue::NativeFn(silk_string_replace, String::from("Replace(s: String, old: String, new: String) -> String; Replaces occurrences of a substring with another substring")));
    
    
    map.insert("reverse".to_string(), SilkValue::NativeFn(silk_string_reverse, String::from("Reverse(s: String) -> String; Returns a reversed version of a string")));
    map.insert("starts_with".to_string(), SilkValue::NativeFn(silk_string_starts_with, String::from("StartsWith(s: String, prefix: String) -> Boolean; Checks if a string starts with a prefix")));
    map.insert("ends_with".to_string(), SilkValue::NativeFn(silk_string_ends_with, String::from("EndsWith(s: String, suffix: String) -> Boolean; Checks if a string ends with a suffix")));
    map.insert("index_of".to_string(), SilkValue::NativeFn(silk_string_index_of, String::from("IndexOf(s: String, needle: String) -> Int; Returns the index of the first occurrence of a substring in a string")));
    map.insert("repeat".to_string(), SilkValue::NativeFn(silk_string_repeat, String::from("Repeat(s: String, n: Int) -> String; Returns a string repeated n times")));
    map.insert("char_at".to_string(), SilkValue::NativeFn(silk_string_char_at, String::from("CharAt(s: String, index: Int) -> String; Returns the character at a specific index in a string")));
    map.insert("count".to_string(), SilkValue::NativeFn(silk_string_count, String::from("Count(s: String, needle: String) -> Int; Counts the occurrences of a substring in a string")));
    map
}