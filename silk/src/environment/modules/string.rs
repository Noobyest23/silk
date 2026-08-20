use std::collections::HashMap;
use crate::environment::vm::{SilkHandle, VirtualMachine};
use super::super::value::SilkValue;

// @export Modules/String
/*
    The String module provides functions for string manipulation, inspection, transformations, and substring searches.
*/

// @export Modules/String#len
/*
    <b>Signature</b>
    <code>len(s: String) -> Int</code>

    <p>Returns the total number of bytes/characters in a string.</p>

    <b>Parameters:</b>
    - <code>s</code>: Target string to inspect.

    <b>Returns:</b>
    - <code>Int</code>: Total length of the string.

    <b>Usage:</b>
    <pre><code>var length = "Hello World".len() # 11</code></pre>
*/
pub fn silk_string_len(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'len' expects exactly 1 argument"));
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    SilkValue::Int(s.len() as i32)
}

// @export Modules/String#concat
/*
    <b>Signature</b>
    <code>concat(s1: String, s2: String) -> String</code>

    <p>Concatenates two strings together and returns the result.</p>

    <b>Parameters:</b>
    - <code>s1</code>: First string segment.
    - <code>s2</code>: Second string segment.

    <b>Returns:</b>
    - <code>String</code>: Combined string value.

    <b>Usage:</b>
    <pre><code>var greeting = "Hello ".concat("World") # "Hello World"</code></pre>
*/
pub fn silk_string_concat(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'concat' expects exactly 2 arguments"));
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

// @export Modules/String#upper
/*
    <b>Signature</b>
    <code>upper(s: String) -> String</code>

    <p>Converts all characters in a string to uppercase.</p>

    <b>Parameters:</b>
    - <code>s</code>: Input string to transform.

    <b>Returns:</b>
    - <code>String</code>: Uppercase string representation.

    <b>Usage:</b>
    <pre><code>var loud = "hello".upper() # "HELLO"</code></pre>
*/
pub fn silk_string_upper(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'upper' expects exactly 1 argument"));
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

// @export Modules/String#lower
/*
    <b>Signature</b>
    <code>lower(s: String) -> String</code>

    <p>Converts all characters in a string to lowercase.</p>

    <b>Parameters:</b>
    - <code>s</code>: Input string to transform.

    <b>Returns:</b>
    - <code>String</code>: Lowercase string representation.

    <b>Usage:</b>
    <pre><code>var quiet = "WORLD".lower() # "world"</code></pre>
*/
pub fn silk_string_lower(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'lower' expects exactly 1 argument"));
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

// @export Modules/String#substring
/*
    <b>Signature</b>
    <code>substring(s: String, start: Int, end: Int) -> String</code>

    <p>Extracts a range of characters from index <code>start</code> up to, but excluding, <code>end</code>.</p>

    <b>Parameters:</b>
    - <code>s</code>: Input string.
    - <code>start</code>: Zero-based starting character index.
    - <code>end</code>: Zero-based ending character index (exclusive).

    <b>Returns:</b>
    - <code>String</code>: Extracted sub-string segment.

    <b>Usage:</b>
    <pre><code>var sub = "Hello World".substring(0, 5) # "Hello"</code></pre>
*/
pub fn silk_string_substring(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 3 {
        vm.error(String::from("'substring' expects exactly 3 arguments"));
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let SilkValue::Int(start) = args[1] else {
        vm.error(String::from("'substring' argument 2 must be an integer (start index)"));
        return SilkValue::Null;
    };

    let SilkValue::Int(end) = args[2] else {
        vm.error(String::from("'substring' argument 3 must be an integer (end index)"));
        return SilkValue::Null;
    };

    let start_idx = (start.max(0) as usize).min(s.len());
    let end_idx = (end.max(0) as usize).min(s.len());

    if start_idx > end_idx {
        vm.error(String::from("'substring' start index cannot be greater than end index"));
        return SilkValue::Null;
    }

    let sub_str: String = s.chars().skip(start_idx).take(end_idx - start_idx).collect();

    let handle = vm.heap_allocate(SilkValue::String(sub_str));
    match handle {
        SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
        _ => unreachable!(),
    }
}

// @export Modules/String#trim
/*
    <b>Signature</b>
    <code>trim(s: String) -> String</code>

    <p>Strips leading and trailing whitespace characters from a string.</p>

    <b>Parameters:</b>
    - <code>s</code>: Input string to trim.

    <b>Returns:</b>
    - <code>String</code>: Cleaned string without boundary whitespace.

    <b>Usage:</b>
    <pre><code>var clean = "   hello   ".trim() # "hello"</code></pre>
*/
pub fn silk_string_trim(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'trim' expects exactly 1 argument"));
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

// @export Modules/String#contains
/*
    <b>Signature</b>
    <code>contains(s: String, needle: String) -> Bool</code>

    <p>Checks whether a target string contains a specified substring sequence.</p>

    <b>Parameters:</b>
    - <code>s</code>: Base string to search within.
    - <code>needle</code>: Target substring to check for.

    <b>Returns:</b>
    - <code>Bool</code>: <code>true</code> if the needle substring is found, otherwise <code>false</code>.

    <b>Usage:</b>
    <pre><code>if "banana".contains("nan") {
    # Match found
}</code></pre>
*/
pub fn silk_string_contains(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'contains' expects exactly 2 arguments"));
        return SilkValue::Null;
    }

    let haystack = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    let needle = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    SilkValue::Bool(haystack.contains(&needle))
}

// @export Modules/String#replace
/*
    <b>Signature</b>
    <code>replace(s: String, old: String, new: String) -> String</code>

    <p>Replaces all occurrences of a target substring with a replacement string.</p>

    <b>Parameters:</b>
    - <code>s</code>: Input string.
    - <code>old</code>: Target substring pattern to find.
    - <code>new</code>: Replacement string content.

    <b>Returns:</b>
    - <code>String</code>: Transformed string value.

    <b>Usage:</b>
    <pre><code>var res = "foo bar".replace("bar", "baz") # "foo baz"</code></pre>
*/
pub fn silk_string_replace(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 3 {
        vm.error(String::from("'replace' expects exactly 3 arguments"));
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

// @export Modules/String#reverse
/*
    <b>Signature</b>
    <code>reverse(s: String) -> String</code>

    <p>Reverses the character sequence of a given string.</p>

    <b>Parameters:</b>
    - <code>s</code>: Input string to reverse.

    <b>Returns:</b>
    - <code>String</code>: Reversed string value.

    <b>Usage:</b>
    <pre><code>var rev = "abc".reverse() # "cba"</code></pre>
*/
pub fn silk_string_reverse(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'reverse' expects exactly 1 argument"));
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

// @export Modules/String#starts_with
/*
    <b>Signature</b>
    <code>starts_with(s: String, prefix: String) -> Bool</code>

    <p>Verifies if a string begins with the specified prefix sequence.</p>

    <b>Parameters:</b>
    - <code>s</code>: Base string to inspect.
    - <code>prefix</code>: Prefix text sequence to test.

    <b>Returns:</b>
    - <code>Bool</code>: <code>true</code> if prefix matches, <code>false</code> otherwise.

    <b>Usage:</b>
    <pre><code>if "http://example.com".starts_with("http") {
    # URL prefix matched
}</code></pre>
*/
pub fn silk_string_starts_with(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'starts_with' expects exactly 2 arguments"));
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    let prefix = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    SilkValue::Bool(s.starts_with(&prefix))
}

// @export Modules/String#ends_with
/*
    <b>Signature</b>
    <code>ends_with(s: String, suffix: String) -> Bool</code>

    <p>Verifies if a string terminates with the specified suffix sequence.</p>

    <b>Parameters:</b>
    - <code>s</code>: Base string to inspect.
    - <code>suffix</code>: Suffix text sequence to test.

    <b>Returns:</b>
    - <code>Bool</code>: <code>true</code> if suffix matches, <code>false</code> otherwise.

    <b>Usage:</b>
    <pre><code>if "image.png".ends_with(".png") {
    # PNG extension matched
}</code></pre>
*/
pub fn silk_string_ends_with(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'ends_with' expects exactly 2 arguments"));
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    let suffix = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    SilkValue::Bool(s.ends_with(&suffix))
}

// @export Modules/String#index_of
/*
    <b>Signature</b>
    <code>index_of(s: String, needle: String) -> Int</code>

    <p>Returns the byte index of the first occurrence of a substring.</p>

    <b>Parameters:</b>
    - <code>s</code>: Base string to search within.
    - <code>needle</code>: Target substring sequence.

    <b>Returns:</b>
    - <code>Int</code>: Zero-based byte index of match start, or <code>-1</code> if not found.

    <b>Usage:</b>
    <pre><code>var idx = "hello world".index_of("world") # 6</code></pre>
*/
pub fn silk_string_index_of(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'index_of' expects exactly 2 arguments"));
        return SilkValue::Null;
    }

    let haystack = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    let needle = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    match haystack.find(&needle) {
        Some(idx) => SilkValue::Int(idx as i32),
        None => SilkValue::Int(-1),
    }
}

// @export Modules/String#repeat
/*
    <b>Signature</b>
    <code>repeat(s: String, n: Int) -> String</code>

    <p>Creates a string by duplicating an input string <code>n</code> times.</p>

    <b>Parameters:</b>
    - <code>s</code>: Text payload to repeat.
    - <code>n</code>: Number of repetitions to generate.

    <b>Returns:</b>
    - <code>String</code>: Repeated string sequence.

    <b>Usage:</b>
    <pre><code>var echo = "ha".repeat(3) # "hahaha"</code></pre>
*/
pub fn silk_string_repeat(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'repeat' expects exactly 2 arguments"));
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let SilkValue::Int(count) = args[1] else {
        vm.error(String::from("'repeat' argument 2 must be an integer (repetition count)"));
        return SilkValue::Null;
    };

    if count < 0 {
        vm.error(String::from("'repeat' count cannot be negative"));
        return SilkValue::Null;
    }

    let repeated = s.repeat(count as usize);
    let handle = vm.heap_allocate(SilkValue::String(repeated));
    match handle {
        SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
        _ => unreachable!(),
    }
}

// @export Modules/String#char_at
/*
    <b>Signature</b>
    <code>char_at(s: String, index: Int) -> String</code>

    <p>Returns a single-character string residing at the specified zero-based character position.</p>

    <b>Parameters:</b>
    - <code>s</code>: Target string.
    - <code>index</code>: Zero-based character index.

    <b>Returns:</b>
    - <code>String</code>: Single-character string, or <code>Null</code> if the index exceeds string boundaries.

    <b>Usage:</b>
    <pre><code>var ch = "Silk".char_at(0) # "S"</code></pre>
*/
pub fn silk_string_char_at(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'char_at' expects exactly 2 arguments"));
        return SilkValue::Null;
    }

    let s = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let SilkValue::Int(idx) = args[1] else {
        vm.error(String::from("'char_at' argument 2 must be an integer (index)"));
        return SilkValue::Null;
    };

    if idx < 0 {
        vm.error(String::from("'char_at' index cannot be negative"));
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

// @export Modules/String#count
/*
    <b>Signature</b>
    <code>count(s: String, needle: String) -> Int</code>

    <p>Counts the non-overlapping occurrences of a substring inside a target string.</p>

    <b>Parameters:</b>
    - <code>s</code>: Base string to search within.
    - <code>needle</code>: Target substring pattern.

    <b>Returns:</b>
    - <code>Int</code>: Total match count.

    <b>Usage:</b>
    <pre><code>var total = "banana".count("a") # 3</code></pre>
*/
pub fn silk_string_count(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'count' expects exactly 2 arguments"));
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