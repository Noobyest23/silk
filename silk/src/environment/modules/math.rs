use core::f32;
use std::collections::HashMap;
use crate::environment::vm::VirtualMachine;
use super::super::value::SilkValue;

fn get_double(val: &SilkValue) -> Option<f64> {
    match val {
        SilkValue::Float(f) => Some(*f as f64),
        SilkValue::Int(i) => Some(*i as f64),
        _ => None,
    }
}

pub fn silk_math_abs(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'abs' expected exactly 1 argument"));
        return SilkValue::Null;
    }

    match &args[0] {
        SilkValue::Int(i) => SilkValue::Int(i.abs()),
        SilkValue::Float(f) => SilkValue::Float(f.abs()),
        _ => {
            vm.error(String::from("'abs' argument must be a number"));
            SilkValue::Null
        }
    }
}

pub fn silk_math_sqrt(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'sqrt' expected exactly 1 argument"));
        return SilkValue::Null;
    }

    if let Some(num) = get_double(&args[0]) {
        SilkValue::Float(num.sqrt() as f32)
    } else {
        vm.error(String::from("'sqrt' argument must be a number"));
        SilkValue::Null
    }
}

pub fn silk_math_pow(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'pow' expected exactly 2 arguments (base, exponent)"));
        return SilkValue::Null;
    }

    let base = get_double(&args[0]);
    let exp = get_double(&args[1]);

    match (base, exp) {
        (Some(b), Some(e)) => SilkValue::Float(b.powf(e) as f32),
        _ => {
            vm.error(String::from("'pow' arguments must be numbers"));
            SilkValue::Null
        }
    }
}

pub fn silk_math_floor(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'floor' expected exactly 1 argument"));
        return SilkValue::Null;
    }

    if let Some(num) = get_double(&args[0]) {
        SilkValue::Int(num.floor() as i32)
    } else {
        vm.error(String::from("'floor' argument must be a number"));
        SilkValue::Null
    }
}

pub fn silk_math_ceil(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'ceil' expected exactly 1 argument"));
        return SilkValue::Null;
    }

    if let Some(num) = get_double(&args[0]) {
        SilkValue::Int(num.ceil() as i32)
    } else {
        vm.error(String::from("'ceil' argument must be a number"));
        SilkValue::Null
    }
}

pub fn silk_math_round(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'round' expected exactly 1 argument"));
        return SilkValue::Null;
    }

    if let Some(num) = get_double(&args[0]) {
        SilkValue::Int(num.round() as i32)
    } else {
        vm.error(String::from("'round' argument must be a number"));
        SilkValue::Null
    }
}

pub fn silk_math_min(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'min' expected exactly 2 arguments"));
        return SilkValue::Null;
    }

    let a = get_double(&args[0]);
    let b = get_double(&args[1]);

    match (a, b) {
        (Some(x), Some(y)) => {
            if x <= y {
                args[0].clone()
            } else {
                args[1].clone()
            }
        }
        _ => {
            vm.error(String::from("'min' arguments must be numbers"));
            SilkValue::Null
        }
    }
}

pub fn silk_math_max(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'max' expected exactly 2 arguments"));
        return SilkValue::Null;
    }

    let a = get_double(&args[0]);
    let b = get_double(&args[1]);

    match (a, b) {
        (Some(x), Some(y)) => {
            if x >= y {
                args[0].clone()
            } else {
                args[1].clone()
            }
        }
        _ => {
            vm.error(String::from("'max' arguments must be numbers"));
            SilkValue::Null
        }
    }
}

pub fn silk_math_sin(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'sin' expected exactly 1 argument"));
        return SilkValue::Null;
    }

    if let Some(num) = get_double(&args[0]) {
        SilkValue::Float(num.sin() as f32)
    } else {
        vm.error(String::from("'sin' argument must be a number"));
        SilkValue::Null
    }
}

pub fn silk_math_cos(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'cos' expected exactly 1 argument"));
        return SilkValue::Null;
    }

    if let Some(num) = get_double(&args[0]) {
        SilkValue::Float(num.cos() as f32)
    } else {
        vm.error(String::from("'cos' argument must be a number"));
        SilkValue::Null
    }
}

pub fn silk_math_tan(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'tan' expected exactly 1 argument"));
        return SilkValue::Null;
    }

    if let Some(num) = get_double(&args[0]) {
        SilkValue::Float(num.tan() as f32)
    } else {
        vm.error(String::from("'tan' argument must be a number"));
        SilkValue::Null
    }
}

pub fn build_math_map() -> HashMap<String, SilkValue> {
    let mut map = HashMap::new();
    
    map.insert("abs".to_string(), SilkValue::NativeFn(silk_math_abs, String::from("Abs(num: Number) -> Number; Returns the absolute value of a number")));
    map.insert("sqrt".to_string(), SilkValue::NativeFn(silk_math_sqrt, String::from("Sqrt(num: Number) -> Number; Returns the square root of a number")));
    map.insert("pow".to_string(), SilkValue::NativeFn(silk_math_pow, String::from("Pow(base: Number, exponent: Number) -> Number; Returns the base raised to the power of the exponent")));
    
    map.insert("floor".to_string(), SilkValue::NativeFn(silk_math_floor, String::from("Floor(num: Number) -> Number; Returns the largest integer less than or equal to a number")));
    map.insert("ceil".to_string(), SilkValue::NativeFn(silk_math_ceil, String::from("Ceil(num: Number) -> Number; Returns the smallest integer greater than or equal to a number")));
    map.insert("round".to_string(), SilkValue::NativeFn(silk_math_round, String::from("Round(num: Number) -> Number; Returns the nearest integer to a number")));
    
    map.insert("min".to_string(), SilkValue::NativeFn(silk_math_min, String::from("Min(num1: Number, num2: Number) -> Number; Returns the smaller of two numbers")));
    map.insert("max".to_string(), SilkValue::NativeFn(silk_math_max, String::from("Max(num1: Number, num2: Number) -> Number; Returns the larger of two numbers")));
    
    map.insert("sin".to_string(), SilkValue::NativeFn(silk_math_sin, String::from("Sin(num: Number) -> Number; Returns the sine of a number")));
    map.insert("cos".to_string(), SilkValue::NativeFn(silk_math_cos, String::from("Cos(num: Number) -> Number; Returns the cosine of a number")));
    map.insert("tan".to_string(), SilkValue::NativeFn(silk_math_tan, String::from("Tan(num: Number) -> Number; Returns the tangent of a number")));
    
    map.insert("PI".to_string(), SilkValue::Float(std::f64::consts::PI as f32));
    map.insert("E".to_string(), SilkValue::Float(std::f64::consts::E as f32));
    
    map
}