use std::collections::HashMap;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::environment::vm::VirtualMachine;
use super::super::value::SilkValue;

fn get_double(val: &SilkValue) -> Option<f64> {
    match val {
        SilkValue::Float(f) => Some(*f as f64),
        SilkValue::Int(i) => Some(*i as f64),
        _ => None,
    }
}

// Returns current UNIX timestamp in seconds as a Float
pub fn silk_time_now(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if !args.is_empty() {
        vm.error("time() expects no arguments".to_string());
        return SilkValue::Null;
    }

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => SilkValue::Float(duration.as_secs_f32()),
        Err(_) => {
            vm.error("System time is set before UNIX epoch".to_string());
            SilkValue::Null
        }
    }
}

// Returns current UNIX timestamp in milliseconds as an Int
pub fn silk_time_ms(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if !args.is_empty() {
        vm.error("time_ms() expects no arguments".to_string());
        return SilkValue::Null;
    }

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => SilkValue::Int(duration.as_millis() as i32),
        Err(_) => {
            vm.error("System time is set before UNIX epoch".to_string());
            SilkValue::Null
        }
    }
}

// Pauses execution for a specified duration in seconds
pub fn silk_time_sleep(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error("sleep(seconds) expects exactly 1 argument".to_string());
        return SilkValue::Null;
    }

    if let Some(secs) = get_double(&args[0]) {
        if secs < 0.0 {
            vm.error("sleep duration cannot be negative".to_string());
            return SilkValue::Null;
        }
        thread::sleep(Duration::from_secs_f64(secs));
        SilkValue::Null
    } else {
        vm.error("sleep duration must be a number".to_string());
        SilkValue::Null
    }
}

pub fn build_time_map() -> HashMap<String, SilkValue> {
    let mut map = HashMap::new();

    map.insert(
        "time".to_string(),
        SilkValue::NativeFn(
            silk_time_now,
            String::from("time() -> Float; Returns current UNIX timestamp in seconds."),
        ),
    );
    map.insert(
        "time_ms".to_string(),
        SilkValue::NativeFn(
            silk_time_ms,
            String::from("time_ms() -> Int; Returns current UNIX timestamp in milliseconds."),
        ),
    );
    map.insert(
        "sleep".to_string(),
        SilkValue::NativeFn(
            silk_time_sleep,
            String::from("sleep(seconds: Number); Pauses program execution for specified duration."),
        ),
    );

    map
}