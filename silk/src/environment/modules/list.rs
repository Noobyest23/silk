use std::collections::HashMap;
use crate::environment::vm::{SilkHandle, VirtualMachine};
use super::super::value::SilkValue;



pub fn silk_list_len(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        eprintln!("[Silk Error] 'len' expects exactly 1 argument");
        return SilkValue::Null;
    }

    let list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    SilkValue::Int(list.len() as i32)
}



pub fn silk_list_contains(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'contains' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    let needle = args[1].clone();

    for item in list {
        
        let matches = match (&item, &needle) {
            (SilkValue::Pointer(_), SilkValue::Pointer(_)) => {
                let item_str = vm.heap_get_string(item.clone());
                let needle_str = vm.heap_get_string(needle.clone());
                match (item_str, needle_str) {
                    (Some(is), Some(ns)) => is == ns,
                    _ => item.equals(&needle)
                }
            }
            _ => item.equals(&needle)
        };
        
        if matches {
            return SilkValue::Bool(true);
        }
    }

    SilkValue::Bool(false)
}



pub fn silk_list_index_of(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'index_of' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    let needle = args[1].clone();

    for (idx, item) in list.iter().enumerate() {
        
        let matches = match (item, &needle) {
            (SilkValue::Pointer(_), SilkValue::Pointer(_)) => {
                let item_str = vm.heap_get_string(item.clone());
                let needle_str = vm.heap_get_string(needle.clone());
                match (item_str, needle_str) {
                    (Some(is), Some(ns)) => is == ns,
                    _ => item.equals(&needle)
                }
            }
            _ => item.equals(&needle)
        };
        
        if matches {
            return SilkValue::Int(idx as i32);
        }
    }

    SilkValue::Int(-1)
}



pub fn silk_list_slice(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 3 {
        eprintln!("[Silk Error] 'slice' expects exactly 3 arguments");
        return SilkValue::Null;
    }

    let list = vm.heap_get_list(args[0].clone()).unwrap_or_default();

    let SilkValue::Int(start) = args[1] else {
        eprintln!("[Silk Error] 'slice' argument 2 must be an integer (start index)");
        return SilkValue::Null;
    };

    let SilkValue::Int(end) = args[2] else {
        eprintln!("[Silk Error] 'slice' argument 3 must be an integer (end index)");
        return SilkValue::Null;
    };

    let start_idx = (start.max(0) as usize).min(list.len());
    let end_idx = (end.max(0) as usize).min(list.len());
    let sliced = if start_idx <= end_idx {
        list[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    let handle = vm.heap_allocate(SilkValue::List(sliced));
    match handle {
        SilkHandle::HeapAllocated(ptr) => SilkValue::Pointer(ptr),
        _ => unreachable!(),
    }
}



pub fn silk_list_push(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'push' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let mut list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    list.push(args[1].clone());

    let handle = vm.heap_allocate(SilkValue::List(list));
    match handle {
        SilkHandle::HeapAllocated(ptr) => SilkValue::Pointer(ptr),
        _ => unreachable!(),
    }
}



pub fn silk_list_pop(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        eprintln!("[Silk Error] 'pop' expects exactly 1 argument");
        return SilkValue::Null;
    }

    let mut list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    if !list.is_empty() {
        list.pop();
    }

    let handle = vm.heap_allocate(SilkValue::List(list));
    match handle {
        SilkHandle::HeapAllocated(ptr) => SilkValue::Pointer(ptr),
        _ => unreachable!(),
    }
}



pub fn silk_list_first(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        eprintln!("[Silk Error] 'first' expects exactly 1 argument");
        return SilkValue::Null;
    }

    let list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    if !list.is_empty() {
        list[0].clone()
    } else {
        SilkValue::Null
    }
}



pub fn silk_list_last(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        eprintln!("[Silk Error] 'last' expects exactly 1 argument");
        return SilkValue::Null;
    }

    let list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    if !list.is_empty() {
        list[list.len() - 1].clone()
    } else {
        SilkValue::Null
    }
}



pub fn silk_list_reverse(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        eprintln!("[Silk Error] 'reverse' expects exactly 1 argument");
        return SilkValue::Null;
    }

    let mut list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    list.reverse();

    let handle = vm.heap_allocate(SilkValue::List(list));
    match handle {
        SilkHandle::HeapAllocated(ptr) => SilkValue::Pointer(ptr),
        _ => unreachable!(),
    }
}



pub fn silk_list_count(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        eprintln!("[Silk Error] 'count' expects exactly 2 arguments");
        return SilkValue::Null;
    }

    let list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    let needle = args[1].clone();

    let mut count = 0;
    for item in list.iter() {
        
        let matches = match (item, &needle) {
            (SilkValue::Pointer(_), SilkValue::Pointer(_)) => {
                let item_str = vm.heap_get_string(item.clone());
                let needle_str = vm.heap_get_string(needle.clone());
                match (item_str, needle_str) {
                    (Some(is), Some(ns)) => is == ns,
                    _ => item.equals(&needle)
                }
            }
            _ => item.equals(&needle)
        };
        
        if matches {
            count += 1;
        }
    }
    SilkValue::Int(count)
}

pub fn build_list_map() -> HashMap<String, SilkValue> {
    let mut map = HashMap::new();
    map.insert("len".to_string(), SilkValue::NativeFn(silk_list_len));
    map.insert("contains".to_string(), SilkValue::NativeFn(silk_list_contains));
    map.insert("index_of".to_string(), SilkValue::NativeFn(silk_list_index_of));
    map.insert("slice".to_string(), SilkValue::NativeFn(silk_list_slice));
    map.insert("push".to_string(), SilkValue::NativeFn(silk_list_push));
    map.insert("pop".to_string(), SilkValue::NativeFn(silk_list_pop));
    map.insert("first".to_string(), SilkValue::NativeFn(silk_list_first));
    map.insert("last".to_string(), SilkValue::NativeFn(silk_list_last));
    map.insert("reverse".to_string(), SilkValue::NativeFn(silk_list_reverse));
    map.insert("count".to_string(), SilkValue::NativeFn(silk_list_count));
    map
}
