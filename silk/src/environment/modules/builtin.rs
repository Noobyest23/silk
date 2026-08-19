use std::collections::HashMap;

use crate::environment::{value::SilkValue, vm::VirtualMachine};

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

pub fn build_builtin_map() -> HashMap<String, SilkValue> {
    let mut map = HashMap::new();
    map.insert(
    "range".to_string(),
    SilkValue::NativeFn(
        silk_builtin_range,
        String::from("range(start_or_stop: Int, stop: Int = null, step: Int = 1) -> Array; Generates an array of integers within a specified interval."),
    ),
);
    map
}