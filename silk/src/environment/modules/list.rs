use std::collections::HashMap;
use crate::environment::vm::{SilkHandle, VirtualMachine};
use super::super::value::SilkValue;

// @export Modules/List
/*
    The List module provides functions for querying, modifying, and manipulating list structures.
*/

// @export Modules/List#len
/*
    <b>Signature</b>
    <code>len(list: List) -> Int</code>

    <p>Returns the total number of elements contained in the specified list.</p>

    <b>Parameters:</b>
    - <code>list</code>: The target list to inspect.

    <b>Returns:</b>
    - <code>Int</code>: The number of items in the list.

    <b>Usage:</b>
    <pre><code>var items = [1, 2, 3]
var count = items.len() # 3</code></pre>
*/
pub fn silk_list_len(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'len' expects exactly 1 argument"));
    }

    let list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    SilkValue::Int(list.len() as i32)
}

// @export Modules/List#contains
/*
    <b>Signature</b>
    <code>contains(list: List, item: Any) -> Bool</code>

    <p>Checks whether a given item is present inside the specified list.</p>

    <b>Parameters:</b>
    - <code>list</code>: The list to search through.
    - <code>item</code>: The element or value to search for.

    <b>Returns:</b>
    - <code>Bool</code>: <code>true</code> if the item is found, <code>false</code> otherwise.

    <b>Usage:</b>
    <pre><code>var fruits = ["apple", "banana"]
if fruits.contains("apple") {
    print("Found!")
}</code></pre>
*/
pub fn silk_list_contains(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'contains' expects exactly 2 argument"));
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

// @export Modules/List#index_of
/*
    <b>Signature</b>
    <code>index_of(list: List, item: Any) -> Int</code>

    <p>Finds the zero-based index of the first occurrence of a value in the list.</p>

    <b>Parameters:</b>
    - <code>list</code>: The list to search.
    - <code>item</code>: The target element to locate.

    <b>Returns:</b>
    - <code>Int</code>: Zero-based index of the item, or <code>-1</code> if not found.

    <b>Usage:</b>
    <pre><code>var letters = ["a", "b", "c"]
var idx = letters.index_of("b") # 1</code></pre>
*/
pub fn silk_list_index_of(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'index_of' expects exactly 2 argument"));
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

// @export Modules/List#slice
/*
    <b>Signature</b>
    <code>slice(list: List, start: Int, end: Int) -> List</code>

    <p>Creates and returns a new list containing elements from index <code>start</code> up to, but excluding, index <code>end</code>.</p>

    <b>Parameters:</b>
    - <code>list</code>: Source list to slice.
    - <code>start</code>: Zero-based starting index.
    - <code>end</code>: Zero-based ending index (exclusive).

    <b>Returns:</b>
    - <code>List</code>: A sub-list created from the target range.

    <b>Usage:</b>
    <pre><code>var nums = [10, 20, 30, 40]
var sub = nums.slice(1, 3) # [20, 30]</code></pre>
*/
pub fn silk_list_slice(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 3 {
        vm.error(String::from("'slice' expects exactly 3 argument"));
    }

    let list = vm.heap_get_list(args[0].clone()).unwrap_or_default();

    let SilkValue::Int(start) = args[1] else {
        vm.error(String::from("'slice' argument 2 must be an integer (start index)"));
        unreachable!();
    };

    let SilkValue::Int(end) = args[2] else {
        vm.error(String::from("'slice' argument 3 must be an integer (end index)"));
        unreachable!();
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

// @export Modules/List#push
/*
    <b>Signature</b>
    <code>push(list: List, item: Any) -> List</code>

    <p>Appends an item to the end of a list and returns an updated list pointer.</p>

    <b>Parameters:</b>
    - <code>list</code>: Target list to expand.
    - <code>item</code>: Value to append.

    <b>Returns:</b>
    - <code>List</code>: Pointer to the newly allocated list with the element added.

    <b>Usage:</b>
    <pre><code>var items = [1, 2]
items = items.push(3)</code></pre>
*/
pub fn silk_list_push(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'push' expects exactly 2 arguments"));
    }

    let mut list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    list.push(args[1].clone());

    let handle = vm.heap_allocate(SilkValue::List(list));
    match handle {
        SilkHandle::HeapAllocated(ptr) => SilkValue::Pointer(ptr),
        _ => unreachable!(),
    }
}

// @export Modules/List#pop
/*
    <b>Signature</b>
    <code>pop(list: List) -> List</code>

    <p>Removes the last element from a list and returns an updated list pointer.</p>

    <b>Parameters:</b>
    - <code>list</code>: Target list to shorten.

    <b>Returns:</b>
    - <code>List</code>: Pointer to the newly allocated list with the last element removed.

    <b>Usage:</b>
    <pre><code>var items = [1, 2, 3]
items = items.pop()</code></pre>
*/
pub fn silk_list_pop(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'pop' expects exactly 1 argument"));
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

// @export Modules/List#first
/*
    <b>Signature</b>
    <code>first(list: List) -> Any</code>

    <p>Retrieves the first element of a list.</p>

    <b>Parameters:</b>
    - <code>list</code>: Target list.

    <b>Returns:</b>
    - <code>Any</code>: The first value in the list, or <code>Null</code> if the list is empty.

    <b>Usage:</b>
    <pre><code>var nums = [10, 20, 30]
var head = nums.first() # 10</code></pre>
*/
pub fn silk_list_first(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'first' expects exactly 1 argument"));
    }

    let list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    if !list.is_empty() {
        list[0].clone()
    } else {
        SilkValue::Null
    }
}

// @export Modules/List#last
/*
    <b>Signature</b>
    <code>last(list: List) -> Any</code>

    <p>Retrieves the final element of a list.</p>

    <b>Parameters:</b>
    - <code>list</code>: Target list.

    <b>Returns:</b>
    - <code>Any</code>: The last value in the list, or <code>Null</code> if the list is empty.

    <b>Usage:</b>
    <pre><code>var nums = [10, 20, 30]
var tail = nums.last() # 30</code></pre>
*/
pub fn silk_list_last(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'last' expects exactly 1 argument"));
    }

    let list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    if !list.is_empty() {
        list[list.len() - 1].clone()
    } else {
        SilkValue::Null
    }
}

// @export Modules/List#reverse
/*
    <b>Signature</b>
    <code>reverse(list: List) -> List</code>

    <p>Reverses the order of elements in a list and returns a new list pointer containing the reversed elements.</p>

    <b>Parameters:</b>
    - <code>list</code>: Target list to reverse.

    <b>Returns:</b>
    - <code>List</code>: Pointer to a new list containing elements in reverse order.

    <b>Usage:</b>
    <pre><code>var nums = [1, 2, 3]
var inverted = nums.reverse() # [3, 2, 1]</code></pre>
*/
pub fn silk_list_reverse(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'reverse' expects exactly 1 argument"));
    }

    let mut list = vm.heap_get_list(args[0].clone()).unwrap_or_default();
    list.reverse();

    let handle = vm.heap_allocate(SilkValue::List(list));
    match handle {
        SilkHandle::HeapAllocated(ptr) => SilkValue::Pointer(ptr),
        _ => unreachable!(),
    }
}

// @export Modules/List#count
/*
    <b>Signature</b>
    <code>count(list: List, item: Any) -> Int</code>

    <p>Counts the total occurrences of a specific item within the list.</p>

    <b>Parameters:</b>
    - <code>list</code>: Target list to examine.
    - <code>item</code>: Value to count within the list.

    <b>Returns:</b>
    - <code>Int</code>: The number of times the item appears.

    <b>Usage:</b>
    <pre><code>var values = [1, 2, 2, 3, 2]
var c = values.count(2) # 3</code></pre>
*/
pub fn silk_list_count(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'count' expects exactly 2 arguments"));
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
    map.insert("len".to_string(), SilkValue::NativeFn(silk_list_len, String::from("len(list: List) -> Int; Returns the length of a list")));
    map.insert("contains".to_string(), SilkValue::NativeFn(silk_list_contains, String::from("contains(list: List, item: Any) -> Boolean; Checks if a list contains an item")));
    map.insert("index_of".to_string(), SilkValue::NativeFn(silk_list_index_of, String::from("index_of(list: List, item: Any) -> Int; Returns the index of an item in a list")));
    map.insert("slice".to_string(), SilkValue::NativeFn(silk_list_slice, String::from("slice(list: List, start: Int, end: Int) -> List; Returns a slice of a list")));
    map.insert("push".to_string(), SilkValue::NativeFn(silk_list_push, String::from("push(list: List, item: Any) -> Null; Adds an item to the end of a list")));
    map.insert("pop".to_string(), SilkValue::NativeFn(silk_list_pop, String::from("pop(list: List) -> Any; Removes and returns the last item from a list")));
    map.insert("first".to_string(), SilkValue::NativeFn(silk_list_first, String::from("first(list: List) -> Any; Returns the first item from a list")));
    map.insert("last".to_string(), SilkValue::NativeFn(silk_list_last, String::from("last(list: List) -> Any; Returns the last item from a list")));
    map.insert("reverse".to_string(), SilkValue::NativeFn(silk_list_reverse, String::from("reverse(list: List) -> Null; Reverses a list")));
    map.insert("count".to_string(), SilkValue::NativeFn(silk_list_count, String::from("count(list: List, item: Any) -> Int; Counts the occurrences of an item in a list")));
    map
}