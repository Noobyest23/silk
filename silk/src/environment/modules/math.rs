use core::f32;
use std::collections::HashMap;
use crate::environment::vm::VirtualMachine;
use super::super::value::SilkValue;

// @export Modules/Math
/*
    The Math module provides basic mathematical constants, operations, geometric utilities, and 2D/3D/4D vector types.
*/

fn get_double(val: &SilkValue) -> Option<f64> {
    match val {
        SilkValue::Float(f) => Some(*f as f64),
        SilkValue::Int(i) => Some(*i as f64),
        _ => None,
    }
}

fn extract_vector_fields(vm: &mut VirtualMachine, self_val: &SilkValue, fields: &[&str]) -> Option<Vec<f32>> {
    let obj_val = match self_val {
        SilkValue::Pointer(ptr) => vm.heap.get(ptr)?,
        val => val,
    };

    if let SilkValue::Object(map) = obj_val {
        let mut results = Vec::new();
        for field in fields {
            let val = map.get(*field)?;
            let num = get_double(val)? as f32;
            results.push(num);
        }
        Some(results)
    } else {
        None
    }
}

// @export Modules/Math#Vector.magnitude
/*
    <b>Signature</b>
    <code>Vector.magnitude() -> Float</code>

    <p>Calculates and returns the magnitude (length) of the vector object.</p>

    <b>Returns:</b>
    - <code>Float</code>: The scalar length of the vector.
*/
pub fn silk_vec_magnitude(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.is_empty() {
        vm.error("Vector magnitude method requires self argument".to_string());
        return SilkValue::Null;
    }

    let fields = if let Some(components) = extract_vector_fields(vm, &args[0], &["x", "y", "z", "w"]) {
        components
    } else if let Some(components) = extract_vector_fields(vm, &args[0], &["x", "y", "z"]) {
        components
    } else if let Some(components) = extract_vector_fields(vm, &args[0], &["x", "y"]) {
        components
    } else {
        vm.error("magnitude() called on invalid Vector instance".to_string());
        return SilkValue::Null;
    };

    let sum_sq: f32 = fields.iter().map(|c| c * c).sum();
    SilkValue::Float(sum_sq.sqrt())
}

// @export Modules/Math#Vector.dot
/*
    <b>Signature</b>
    <code>Vector.dot(other: Vector) -> Float</code>

    <p>Calculates the scalar dot product between the current vector and another matching dimension vector.</p>

    <b>Parameters:</b>
    - <code>other</code>: Vector object of matching dimension.

    <b>Returns:</b>
    - <code>Float</code>: The scalar dot product.
*/
pub fn silk_vec_dot(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() < 2 {
        vm.error("dot() expects self and another vector as arguments".to_string());
        return SilkValue::Null;
    }

    let fields = ["x", "y", "z", "w"];
    for len in (2..=4).rev() {
        let active_fields = &fields[..len];
        if let (Some(v1), Some(v2)) = (
            extract_vector_fields(vm, &args[0], active_fields),
            extract_vector_fields(vm, &args[1], active_fields),
        ) {
            let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
            return SilkValue::Float(dot_product);
        }
    }

    vm.error("dot() requires two vectors of matching dimension".to_string());
    SilkValue::Null
}

// @export Modules/Math#Vec2
/*
    <b>Signature</b>
    <code>Vec2(x: Number = 0, y: Number = 0) -> Vector2</code>

    <p>Constructs a 2D vector object containing x and y components alongside vector arithmetic methods.</p>

    <b>Parameters:</b>
    - <code>x</code>: (Optional) X coordinate component.
    - <code>y</code>: (Optional) Y coordinate component.

    <b>Returns:</b>
    - <code>Vector2</code>: A 2D vector instance.

    <b>Usage:</b>
    <pre><code>var v = Vec2(3, 4)
var len = v.magnitude() # 5.0</code></pre>
*/
pub fn silk_vector2(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    let x = if args.len() > 0 { get_double(&args[0]).unwrap_or(0.0) as f32 } else { 0.0 };
    let y = if args.len() > 1 { get_double(&args[1]).unwrap_or(0.0) as f32 } else { 0.0 };

    let mut vec_obj = HashMap::new();
    vec_obj.insert("x".to_string(), SilkValue::Float(x));
    vec_obj.insert("y".to_string(), SilkValue::Float(y));
    vec_obj.insert("magnitude".to_string(), SilkValue::NativeFn(silk_vec_magnitude, "Vector2.magnitude() -> Float".to_string()));
    vec_obj.insert("dot".to_string(), SilkValue::NativeFn(silk_vec_dot, "Vector2.dot(other: Vector2) -> Float".to_string()));

    let ptr = vm.next_heap_ptr;
    vm.heap_allocate(SilkValue::Object(vec_obj));
    SilkValue::Pointer(ptr)
}

// @export Modules/Math#Vec3
/*
    <b>Signature</b>
    <code>Vec3(x: Number = 0, y: Number = 0, z: Number = 0) -> Vector3</code>

    <p>Constructs a 3D vector object containing x, y, and z components alongside vector arithmetic methods.</p>

    <b>Parameters:</b>
    - <code>x</code>: (Optional) X coordinate component.
    - <code>y</code>: (Optional) Y coordinate component.
    - <code>z</code>: (Optional) Z coordinate component.

    <b>Returns:</b>
    - <code>Vector3</code>: A 3D vector instance.

    <b>Usage:</b>
    <pre><code>var v = Vec3(1, 2, 2)
var len = v.magnitude() # 3.0</code></pre>
*/
pub fn silk_vector3(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    let x = if args.len() > 0 { get_double(&args[0]).unwrap_or(0.0) as f32 } else { 0.0 };
    let y = if args.len() > 1 { get_double(&args[1]).unwrap_or(0.0) as f32 } else { 0.0 };
    let z = if args.len() > 2 { get_double(&args[2]).unwrap_or(0.0) as f32 } else { 0.0 };

    let mut vec_obj = HashMap::new();
    vec_obj.insert("x".to_string(), SilkValue::Float(x));
    vec_obj.insert("y".to_string(), SilkValue::Float(y));
    vec_obj.insert("z".to_string(), SilkValue::Float(z));
    vec_obj.insert("magnitude".to_string(), SilkValue::NativeFn(silk_vec_magnitude, "Vector3.magnitude() -> Float".to_string()));
    vec_obj.insert("dot".to_string(), SilkValue::NativeFn(silk_vec_dot, "Vector3.dot(other: Vector3) -> Float".to_string()));

    let ptr = vm.next_heap_ptr;
    vm.heap_allocate(SilkValue::Object(vec_obj));
    SilkValue::Pointer(ptr)
}

// @export Modules/Math#Vec4
/*
    <b>Signature</b>
    <code>Vec4(x: Number = 0, y: Number = 0, z: Number = 0, w: Number = 0) -> Vector4</code>

    <p>Constructs a 4D vector object containing x, y, z, and w components alongside vector arithmetic methods.</p>

    <b>Parameters:</b>
    - <code>x</code>: (Optional) X coordinate component.
    - <code>y</code>: (Optional) Y coordinate component.
    - <code>z</code>: (Optional) Z coordinate component.
    - <code>w</code>: (Optional) W coordinate component.

    <b>Returns:</b>
    - <code>Vector4</code>: A 4D vector instance.

    <b>Usage:</b>
    <pre><code>var v1 = Vec4(1, 0, 0, 0)
var v2 = Vec4(0, 1, 0, 0)
var d = v1.dot(v2) # 0.0</code></pre>
*/
pub fn silk_vector4(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    let x = if args.len() > 0 { get_double(&args[0]).unwrap_or(0.0) as f32 } else { 0.0 };
    let y = if args.len() > 1 { get_double(&args[1]).unwrap_or(0.0) as f32 } else { 0.0 };
    let z = if args.len() > 2 { get_double(&args[2]).unwrap_or(0.0) as f32 } else { 0.0 };
    let w = if args.len() > 3 { get_double(&args[3]).unwrap_or(0.0) as f32 } else { 0.0 };

    let mut vec_obj = HashMap::new();
    vec_obj.insert("x".to_string(), SilkValue::Float(x));
    vec_obj.insert("y".to_string(), SilkValue::Float(y));
    vec_obj.insert("z".to_string(), SilkValue::Float(z));
    vec_obj.insert("w".to_string(), SilkValue::Float(w));
    vec_obj.insert("magnitude".to_string(), SilkValue::NativeFn(silk_vec_magnitude, "Vector4.magnitude() -> Float".to_string()));
    vec_obj.insert("dot".to_string(), SilkValue::NativeFn(silk_vec_dot, "Vector4.dot(other: Vector4) -> Float".to_string()));

    let ptr = vm.next_heap_ptr;
    vm.heap_allocate(SilkValue::Object(vec_obj));
    SilkValue::Pointer(ptr)
}

// @export Modules/Math#abs
/*
    <b>Signature</b>
    <code>abs(num: Number) -> Number</code>

    <p>Returns the non-negative absolute value of a given number.</p>

    <b>Parameters:</b>
    - <code>num</code>: Numerical input value.

    <b>Returns:</b>
    - <code>Number</code>: Absolute value matching input type (Integer or Float).

    <b>Usage:</b>
    <pre><code>var a = abs(-5) # 5</code></pre>
*/
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

// @export Modules/Math#sqrt
/*
    <b>Signature</b>
    <code>sqrt(num: Number) -> Float</code>

    <p>Computes the square root of a non-negative number.</p>

    <b>Parameters:</b>
    - <code>num</code>: Numerical input value.

    <b>Returns:</b>
    - <code>Float</code>: Calculated square root value.

    <b>Usage:</b>
    <pre><code>var root = sqrt(16) # 4.0</code></pre>
*/
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

// @export Modules/Math#pow
/*
    <b>Signature</b>
    <code>pow(base: Number, exponent: Number) -> Float</code>

    <p>Raises a base number to the specified power exponent.</p>

    <b>Parameters:</b>
    - <code>base</code>: Base number.
    - <code>exponent</code>: Power exponent number.

    <b>Returns:</b>
    - <code>Float</code>: Result of base raised to exponent.

    <b>Usage:</b>
    <pre><code>var res = pow(2, 3) # 8.0</code></pre>
*/
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

// @export Modules/Math#floor
/*
    <b>Signature</b>
    <code>floor(num: Number) -> Int</code>

    <p>Rounds a floating-point number down to the nearest integer lower than or equal to the value.</p>

    <b>Parameters:</b>
    - <code>num</code>: Numerical input value.

    <b>Returns:</b>
    - <code>Int</code>: Rounded integer.

    <b>Usage:</b>
    <pre><code>var val = floor(3.7) # 3</code></pre>
*/
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

// @export Modules/Math#ceil
/*
    <b>Signature</b>
    <code>ceil(num: Number) -> Int</code>

    <p>Rounds a floating-point number up to the nearest integer greater than or equal to the value.</p>

    <b>Parameters:</b>
    - <code>num</code>: Numerical input value.

    <b>Returns:</b>
    - <code>Int</code>: Rounded integer.

    <b>Usage:</b>
    <pre><code>var val = ceil(3.2) # 4</code></pre>
*/
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

// @export Modules/Math#round
/*
    <b>Signature</b>
    <code>round(num: Number) -> Int</code>

    <p>Rounds a floating-point value to the nearest integer.</p>

    <b>Parameters:</b>
    - <code>num</code>: Numerical input value.

    <b>Returns:</b>
    - <code>Int</code>: Nearest rounded integer.

    <b>Usage:</b>
    <pre><code>var val = round(3.5) # 4</code></pre>
*/
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

// @export Modules/Math#min
/*
    <b>Signature</b>
    <code>min(num1: Number, num2: Number) -> Number</code>

    <p>Compares two numbers and returns the smaller value.</p>

    <b>Parameters:</b>
    - <code>num1</code>: First input value.
    - <code>num2</code>: Second input value.

    <b>Returns:</b>
    - <code>Number</code>: The smaller value among the two inputs.

    <b>Usage:</b>
    <pre><code>var lowest = min(10, 5) # 5</code></pre>
*/
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

// @export Modules/Math#max
/*
    <b>Signature</b>
    <code>max(num1: Number, num2: Number) -> Number</code>

    <p>Compares two numbers and returns the larger value.</p>

    <b>Parameters:</b>
    - <code>num1</code>: First input value.
    - <code>num2</code>: Second input value.

    <b>Returns:</b>
    - <code>Number</code>: The larger value among the two inputs.

    <b>Usage:</b>
    <pre><code>var highest = max(10, 5) # 10</code></pre>
*/
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

// @export Modules/Math#sin
/*
    <b>Signature</b>
    <code>sin(num: Number) -> Float</code>

    <p>Calculates the trigonometric sine of an angle given in radians.</p>

    <b>Parameters:</b>
    - <code>num</code>: Angle expressed in radians.

    <b>Returns:</b>
    - <code>Float</code>: Sine result.

    <b>Usage:</b>
    <pre><code>var val = sin(PI / 2) # 1.0</code></pre>
*/
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

// @export Modules/Math#cos
/*
    <b>Signature</b>
    <code>cos(num: Number) -> Float</code>

    <p>Calculates the trigonometric cosine of an angle given in radians.</p>

    <b>Parameters:</b>
    - <code>num</code>: Angle expressed in radians.

    <b>Returns:</b>
    - <code>Float</code>: Cosine result.

    <b>Usage:</b>
    <pre><code>var val = cos(0) # 1.0</code></pre>
*/
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

// @export Modules/Math#tan
/*
    <b>Signature</b>
    <code>tan(num: Number) -> Float</code>

    <p>Calculates the trigonometric tangent of an angle given in radians.</p>

    <b>Parameters:</b>
    - <code>num</code>: Angle expressed in radians.

    <b>Returns:</b>
    - <code>Float</code>: Tangent result.

    <b>Usage:</b>
    <pre><code>var val = tan(0) # 0.0</code></pre>
*/
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
    
    map.insert("Vec2".to_string(), SilkValue::NativeFn(silk_vector2, String::from("Vec2(x: Number = 0, y: Number = 0) -> Vector2")));
    map.insert("Vec3".to_string(), SilkValue::NativeFn(silk_vector3, String::from("Vec3(x: Number = 0, y: Number = 0, z: Number = 0) -> Vector3")));
    map.insert("Vec4".to_string(), SilkValue::NativeFn(silk_vector4, String::from("Vec4(x: Number = 0, y: Number = 0, z: Number = 0, w: Number = 0) -> Vector4")));

    map
}