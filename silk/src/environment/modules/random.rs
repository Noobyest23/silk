use std::collections::HashMap;
use crate::environment::vm::VirtualMachine;
use super::super::value::SilkValue;

// @export Modules/Random
/*
    The Random module provides functions for pseudo-random number generation,
    shuffling collections, and making random choices.
*/

fn get_double(val: &SilkValue) -> Option<f64> {
    match val {
        SilkValue::Float(f) => Some(*f as f64),
        SilkValue::Int(i) => Some(*i as f64),
        _ => None,
    }
}

// Simple xorshift64 pseudo-random number generator state
static mut RNG_STATE: u64 = 88172645463325252;

fn next_u64() -> u64 {
    unsafe {
        let mut x = RNG_STATE;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        RNG_STATE = x;
        x
    }
}

fn random_f64() -> f64 {
    (next_u64() as f64) / (u64::MAX as f64)
}

// @export Modules/Random#random
/*
    <b>Signature</b>
    <code>random() -> Float</code>

    <p>Returns a pseudo-random floating point number in the range [0.0, 1.0).</p>

    <b>Returns:</b>
    - <code>Float</code>: A random float between 0.0 and 1.0.

    <b>Usage:</b>
    <pre><code>var val = random()</code></pre>
*/
pub fn silk_random_random(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if !args.is_empty() {
        vm.error("random() expects no arguments".to_string());
        return SilkValue::Null;
    }

    SilkValue::Float(random_f64() as f32)
}

// @export Modules/Random#randint
/*
    <b>Signature</b>
    <code>randint(min: Int, max: Int) -> Int</code>

    <p>Returns a random integer within the inclusive range [min, max].</p>

    <b>Parameters:</b>
    - <code>min</code>: Lower bound (inclusive).
    - <code>max</code>: Upper bound (inclusive).

    <b>Returns:</b>
    - <code>Int</code>: Random integer in range.

    <b>Usage:</b>
    <pre><code>var dice = randint(1, 6)</code></pre>
*/
pub fn silk_random_randint(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error("randint(min, max) expects exactly 2 arguments".to_string());
        return SilkValue::Null;
    }

    let min = match &args[0] {
        SilkValue::Int(i) => *i,
        _ => {
            vm.error("randint() min argument must be an integer".to_string());
            return SilkValue::Null;
        }
    };

    let max = match &args[1] {
        SilkValue::Int(i) => *i,
        _ => {
            vm.error("randint() max argument must be an integer".to_string());
            return SilkValue::Null;
        }
    };

    if min > max {
        vm.error("randint() min cannot be greater than max".to_string());
        return SilkValue::Null;
    }

    let range = (max - min + 1) as u64;
    let val = min + (next_u64() % range) as i32;

    SilkValue::Int(val)
}

// @export Modules/Random#choice
/*
    <b>Signature</b>
    <code>choice(list: List) -> Any</code>

    <p>Returns a random element selected from a non-empty list.</p>

    <b>Parameters:</b>
    - <code>list</code>: Target list to pick from.

    <b>Returns:</b>
    - <code>Any</code>: Random item from the list.

    <b>Usage:</b>
    <pre><code>var picked = choice(["apple", "banana", "cherry"])</code></pre>
*/
pub fn silk_random_choice(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error("choice(list) expects exactly 1 argument".to_string());
        return SilkValue::Null;
    }

    if let Some(list) = vm.heap_get_list(args[0].clone()) {
        if list.is_empty() {
            vm.error("cannot pick a random choice from an empty list".to_string());
            return SilkValue::Null;
        }
        let index = (next_u64() as usize) % list.len();
        list[index].clone()
    } else {
        vm.error("choice() argument must be a list".to_string());
        return SilkValue::Null;
    }
}

pub fn build_random_map() -> HashMap<String, SilkValue> {
    let mut map = HashMap::new();

    map.insert(
        "random".to_string(),
        SilkValue::NativeFn(
            silk_random_random,
            String::from("random() -> Float; Returns a random float between 0.0 and 1.0."),
        ),
    );
    map.insert(
        "randint".to_string(),
        SilkValue::NativeFn(
            silk_random_randint,
            String::from("randint(min: Int, max: Int) -> Int; Returns a random integer in inclusive range [min, max]."),
        ),
    );
    map.insert(
        "choice".to_string(),
        SilkValue::NativeFn(
            silk_random_choice,
            String::from("choice(list: List) -> Any; Returns a random element from a list."),
        ),
    );

    map
}