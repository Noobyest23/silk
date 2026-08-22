mod lexer;
mod parser;
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_float, c_int};
use std::fs::{read_to_string};
use std::os::raw::c_char;
use std::path::Path;
use lexer::Lexer;

mod environment;
use environment::vm;
use environment::modules::io::build_io_map;
use crate::environment::modules::list::build_list_map;
use crate::environment::modules::math::build_math_map;
use crate::environment::modules::string::build_string_map;
use crate::environment::modules::image::build_image_map;
use crate::environment::modules::builtin::build_builtin_map;
use crate::environment::modules::time::build_time_map;
use crate::environment::modules::random::build_random_map;
use crate::environment::modules::json::build_json_map;
pub use crate::environment::{value::{NativeFn, SilkValue}, vm::VirtualMachine};
use crate::parser::Parser;
use std::sync::{Mutex, OnceLock};

static GLOBAL_VM: OnceLock<Mutex<VirtualMachine>> = OnceLock::new();
static HOST_NATIVE_CALLBACK: OnceLock<Mutex<Option<unsafe extern "C" fn(*const c_char) -> *const c_char>>> = OnceLock::new();

fn invoke_host_native_callback(input: &str) -> String {
    let Some(callback_slot) = HOST_NATIVE_CALLBACK.get() else {
        return String::new();
    };

    let callback = {
        let lock = callback_slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        *lock
    };

    let Some(callback) = callback else {
        return String::new();
    };

    let c_input = CString::new(input).unwrap_or_default();
    let result_ptr = unsafe { callback(c_input.as_ptr()) };

    if result_ptr.is_null() {
        return String::new();
    }

    let result = unsafe { CStr::from_ptr(result_ptr) };
    result.to_str().unwrap_or_default().to_string()
}

pub fn register_module(module_name: &str, values: HashMap<String, SilkValue>) -> bool {
    let Some(vm_mutex) = GLOBAL_VM.get() else {
        eprintln!("[Silk Error] VM was never initialized! Call silk_init() first.");
        return false;
    };

    let mut vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    vm.modules.insert(module_name.to_string(), values);
    true
}

pub fn add_module_value(module_name: &str, key: &str, value: SilkValue) -> bool {
    let Some(vm_mutex) = GLOBAL_VM.get() else {
        eprintln!("[Silk Error] VM was never initialized! Call silk_init() first.");
        return false;
    };

    let mut vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    vm.modules.entry(module_name.to_string())
        .or_insert_with(HashMap::new)
        .insert(key.to_string(), value);
    true
}

pub fn register_module_native_fn(module_name: &str, key: &str, func: NativeFn, description: &str) -> bool {
    add_module_value(module_name, key, SilkValue::NativeFn(func, description.to_string()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn register_module_native_fn_host(
    module_name: *const c_char,
    key: *const c_char,
    callback: unsafe extern "C" fn(*const c_char) -> *const c_char,
    description: *const c_char,
) -> bool {
    if module_name.is_null() || key.is_null() || description.is_null() {
        return false;
    }

    let c_name = unsafe { CStr::from_ptr(module_name) };
    let c_key = unsafe { CStr::from_ptr(key) };
    let c_description = unsafe { CStr::from_ptr(description) };

    match (c_name.to_str(), c_key.to_str(), c_description.to_str()) {
        (Ok(name), Ok(key_name), Ok(description_str)) => {
            let callback_slot = HOST_NATIVE_CALLBACK.get_or_init(|| Mutex::new(None));
            {
                let mut lock = callback_slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                *lock = Some(callback);
            }

            let wrapped: NativeFn = |_, args| {
                let Some(SilkValue::String(value)) = args.first() else {
                    return SilkValue::String(String::new());
                };

                SilkValue::String(invoke_host_native_callback(value.as_str()))
            };

            register_module_native_fn(name, key_name, wrapped, description_str)
        }
        _ => false,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn register_module_string(module_name: *const c_char, key: *const c_char, value: *const c_char) -> bool {
    if module_name.is_null() || key.is_null() || value.is_null() {
        return false;
    }

    let c_name = unsafe { CStr::from_ptr(module_name) };
    let c_key = unsafe { CStr::from_ptr(key) };
    let c_value = unsafe { CStr::from_ptr(value) };

    match (c_name.to_str(), c_key.to_str(), c_value.to_str()) {
        (Ok(name), Ok(key_str), Ok(val)) => {
            add_module_value(name, key_str, SilkValue::String(val.to_string()))
        }
        _ => false,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn register_module_int(module_name: *const c_char, key: *const c_char, value: c_int) -> bool {
    if module_name.is_null() || key.is_null() {
        return false;
    }

    let c_name = unsafe { CStr::from_ptr(module_name) };
    let c_key = unsafe { CStr::from_ptr(key) };

    match (c_name.to_str(), c_key.to_str()) {
        (Ok(name), Ok(key_str)) => add_module_value(name, key_str, SilkValue::Int(value as i32)),
        _ => false,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn register_module_float(module_name: *const c_char, key: *const c_char, value: c_float) -> bool {
    if module_name.is_null() || key.is_null() {
        return false;
    }

    let c_name = unsafe { CStr::from_ptr(module_name) };
    let c_key = unsafe { CStr::from_ptr(key) };

    match (c_name.to_str(), c_key.to_str()) {
        (Ok(name), Ok(key_str)) => add_module_value(name, key_str, SilkValue::Float(value as f32)),
        _ => false,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn register_module_bool(module_name: *const c_char, key: *const c_char, value: c_int) -> bool {
    if module_name.is_null() || key.is_null() {
        return false;
    }

    let c_name = unsafe { CStr::from_ptr(module_name) };
    let c_key = unsafe { CStr::from_ptr(key) };

    match (c_name.to_str(), c_key.to_str()) {
        (Ok(name), Ok(key_str)) => add_module_value(name, key_str, SilkValue::Bool(value != 0)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_module_stores_values_for_imports() {
        unsafe {
            let _ = init();
        }

        let mut module = HashMap::new();
        module.insert("greet".to_string(), SilkValue::String("hello".to_string()));

        assert!(register_module("external_test", module));

        let Some(vm_mutex) = GLOBAL_VM.get() else {
            panic!("VM was not initialized");
        };

        let vm = vm_mutex.lock().expect("VM mutex poisoned");
        let entry = vm.modules.get("external_test").expect("module not registered");
        assert!(matches!(entry.get("greet"), Some(SilkValue::String(value)) if value == "hello"));
    }

    #[test]
    fn register_module_native_fn_registers_callable_value() {
        fn native_stub(_vm: &mut crate::environment::vm::VirtualMachine, _args: &Vec<SilkValue>) -> SilkValue {
            SilkValue::Int(7)
        }

        unsafe {
            let _ = init();
        }

        assert!(register_module_native_fn("external_native", "answer", native_stub, "answers the question"));

        let Some(vm_mutex) = GLOBAL_VM.get() else {
            panic!("VM was not initialized");
        };

        let vm = vm_mutex.lock().expect("VM mutex poisoned");
        let entry = vm.modules.get("external_native").expect("module not registered");
        assert!(matches!(entry.get("answer"), Some(SilkValue::NativeFn(_, desc)) if desc == "answers the question"));
    }
}

// @export #Silk
/*
    <p>Silk is a lightweight scripting runtime for building embeddable programs with a small standard library, dynamic execution, and module-based APIs.</p>

    <b>Overview</b>
    <p>The language includes built-in support for global values, math, string handling, list operations, time utilities, image processing, and file I/O. The runtime exposes a VM-backed API that lets host applications initialize the engine, run scripts, and inspect module state.</p>

    <b>Quick Start</b>
    <pre><code>import "io"

print("Hello from Silk!")</code></pre>

    <b>Core runtime entry points</b>
    - <code>init()</code>: Creates the global VM and prepares the available modules.
    - <code>run(path)</code>: Executes a Silk script file from disk.
    - <code>run_source(source)</code>: Executes a string of Silk source directly.
    - <code>inspect(module)</code>: Prints the values registered in a module.

    <b>Built-in modules</b>
    - <code>io</code>: File reading, writing, and console output
    - <code>math</code>: Numeric and vector helpers
    - <code>string</code>: String transformations and parsing utilities
    - <code>list</code>: Sequence operations and element access
    - <code>image</code>: Image loading, editing, and export helpers
    - <code>time</code>: Timing utilities and sleep operations
*/

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init() -> bool {
    
    let init_result = GLOBAL_VM.set(Mutex::new(VirtualMachine::new())).is_ok();

    if !init_result {
        return init_result;
    }

    let Some(vm_mutex) = GLOBAL_VM.get() else {
        eprintln!("[Silk Error] VM was never initialized! Call silk_init() first.");
        return false;
    };

    let mut vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(), 
    };

    
    vm.modules.insert(String::from("io"), build_io_map());
    vm.modules.insert(String::from("math"), build_math_map());
    vm.modules.insert(String::from("string"), build_string_map());
    vm.modules.insert(String::from("list"), build_list_map());
    vm.modules.insert(String::from("image"), build_image_map());
    vm.modules.insert(String::from("builtin"), build_builtin_map());
    vm.modules.insert(String::from("time"), build_time_map());
    vm.modules.insert(String::from("random"), build_random_map());
    vm.modules.insert(String::from("json"), build_json_map());
    init_result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn run(path_ptr: *const c_char) {
    if path_ptr.is_null() {
        return;
    }

    let Some(vm_mutex) = GLOBAL_VM.get() else {
        eprintln!("[Silk Error] VM was never initialized! Call silk_init() first.");
        return;
    };

    let mut vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let c_str = unsafe { CStr::from_ptr(path_ptr) };

    if let Ok(path_str) = c_str.to_str() {
        let path = Path::new(path_str);
        
        if let Ok(src) = read_to_string(path) {
            let mut lexer = Lexer::new(&src);
            let tokens = lexer.tokenize();
        
            let mut parser = Parser::new(tokens);
            let o_program = parser.parse();

            if let Some(program) = o_program {
                vm.execute(program, false, String::from(path_str));
            }
            else {
                eprintln!("[Silk Error] Could not execute script");
            }

        } else {
            eprintln!("[Silk Error] Could not read file path: {}", path_str);
        }
    }
}

pub fn run_source(source: &str) {
    let Some(vm_mutex) = GLOBAL_VM.get() else {
        eprintln!("[Silk Error] VM was never initialized! Call silk_init() first.");
        return;
    };

    let mut vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let o_program = parser.parse();

    if let Some(program) = o_program {
        vm.execute(program, false, String::from("raw source"));
    }
    else {
        eprintln!("[Silk Error] Could not execute script");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn inspect(module: *const c_char) {
    if module.is_null() {
        return;
    }

    let Some(vm_mutex) = GLOBAL_VM.get() else {
        eprintln!("[Silk Error] VM was never initialized! Call silk_init() first.");
        return;
    };

    let vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(), 
    };

    let c_str = unsafe { CStr::from_ptr(module) };

    if let Ok(path_str) = c_str.to_str() {
        let option = vm.modules.get(path_str);
        if let Some(mod_object) = option{
            eprintln!("Values in the {} module", path_str);
            for (key, val) in mod_object {
                eprintln!("{} : {}", key, val);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_global_int(id: *const c_char, v: c_int) {
    if id.is_null() { return; }

    let Some(vm_mutex) = GLOBAL_VM.get() else { return; };
    let mut vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let c_str = unsafe { CStr::from_ptr(id) };
    if let Ok(id_str) = c_str.to_str() {
        
        vm.globals.insert(id_str.to_string(), SilkValue::Int(v as i32));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_global_float(id: *const c_char, v: c_float) {
    if id.is_null() { return; }

    let Some(vm_mutex) = GLOBAL_VM.get() else { return; };
    let mut vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let c_str = unsafe { CStr::from_ptr(id) };
    if let Ok(id_str) = c_str.to_str() {
        vm.globals.insert(id_str.to_string(), SilkValue::Float(v as f32));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_global_string(id: *const c_char, v: *const c_char) {
    if id.is_null() || v.is_null() { return; }

    let Some(vm_mutex) = GLOBAL_VM.get() else { return; };
    let mut vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let c_id = unsafe { CStr::from_ptr(id) };
    let c_val = unsafe { CStr::from_ptr(v) };

    if let (Ok(id_str), Ok(val_str)) = (c_id.to_str(), c_val.to_str()) {
        vm.globals.insert(id_str.to_string(), SilkValue::String(val_str.to_string()));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_global_bool(id: *const c_char, v: c_int) {
    if id.is_null() { return; }

    let Some(vm_mutex) = GLOBAL_VM.get() else { return; };
    let mut vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let c_str = unsafe { CStr::from_ptr(id) };
    if let Ok(id_str) = c_str.to_str() {
        
        let bool_val = v != 0;
        vm.globals.insert(id_str.to_string(), SilkValue::Bool(bool_val));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_global_int(id: *const c_char) -> c_int {
    if id.is_null() { return 0; }

    let Some(vm_mutex) = GLOBAL_VM.get() else { return 0; };
    let vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let c_str = unsafe { CStr::from_ptr(id) };
    if let Ok(id_str) = c_str.to_str() {
        if let Some(SilkValue::Int(val)) = vm.globals.get(id_str) {
            return *val as c_int;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_global_float(id: *const c_char) -> c_float {
    if id.is_null() { return 0.0; }

    let Some(vm_mutex) = GLOBAL_VM.get() else { return 0.0; };
    let vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let c_str = unsafe { CStr::from_ptr(id) };
    if let Ok(id_str) = c_str.to_str() {
        if let Some(SilkValue::Float(val)) = vm.globals.get(id_str) {
            return *val as c_float;
        }
    }
    0.0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_global_string(id: *const c_char, buffer: *mut c_char, buffer_len: usize) -> bool {
    if id.is_null() || buffer.is_null() || buffer_len == 0 { return false; }

    let Some(vm_mutex) = GLOBAL_VM.get() else { return false; };
    let vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let c_id = unsafe { CStr::from_ptr(id) };
    if let Ok(id_str) = c_id.to_str() {
        match vm.globals.get(id_str) {
            Some(SilkValue::String(val)) => {
                let bytes = val.as_bytes();
                let copy_len = std::cmp::min(bytes.len(), buffer_len - 1);
                unsafe {
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, copy_len);
                    *buffer.add(copy_len) = 0; 
                }
                return true;
            }
            Some(SilkValue::Pointer(ptr)) => {
                if let Some(SilkValue::String(val)) = vm.heap.get(ptr) {
                    let bytes = val.as_bytes();
                    let copy_len = std::cmp::min(bytes.len(), buffer_len - 1);
                    unsafe {
                        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, copy_len);
                        *buffer.add(copy_len) = 0; 
                    }
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_global_bool(id: *const c_char) -> c_int {
    if id.is_null() { return 0; }

    let Some(vm_mutex) = GLOBAL_VM.get() else { return 0; };
    let vm = match vm_mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let c_str = unsafe { CStr::from_ptr(id) };
    if let Ok(id_str) = c_str.to_str() {
        if let Some(SilkValue::Bool(val)) = vm.globals.get(id_str) {
            return if *val { 1 } else { 0 };
        }
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn version() -> c_float {
    return 0.1;
}