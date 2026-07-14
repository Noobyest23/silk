mod lexer;
mod parser;
use std::ffi::{CStr, c_float, c_int};
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
use crate::environment::value::SilkValue;
use crate::parser::Parser;

use std::sync::{Mutex, OnceLock};
use vm::VirtualMachine;


static GLOBAL_VM: OnceLock<Mutex<VirtualMachine>> = OnceLock::new();

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
            let program = parser.parse();

            vm.execute(program, false);

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
    let program = parser.parse();

    vm.execute(program, false);
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