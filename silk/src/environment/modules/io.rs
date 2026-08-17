use std::{collections::HashMap, io::Write};
use crate::environment::vm::{SilkHandle, VirtualMachine};
use std::io;
use super::super::value::SilkValue;


pub fn silk_io_print(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    for arg in args {
        
        print!("{}", vm.stringify_value(arg));
    }
    println!();
    SilkValue::Null
}

pub fn silk_io_read(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    
    if args.len() != 1 {
        vm.error(String::from("'read' expects exactly 1 argument"));
    }

    let path_str = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    
    match std::fs::read_to_string(path_str) {
        Ok(contents) => {
            
            let handle = vm.heap_allocate(SilkValue::String(contents));
            match handle {
                SilkHandle::HeapAllocated(p) => SilkValue::Pointer(p),
                _ => unreachable!(),
            }
        }
        Err(e) => {
            vm.error(format!("'read' unable to read file: {}", e));
            SilkValue::Null
        }
    }
}

pub fn silk_io_write(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'write' expects exactly 2 arguments"));
    }

    let path_str = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let contents = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    let result = std::fs::write(path_str, &contents);
    match result {
        Ok(_) => SilkValue::Null,
        Err(e) => {
            vm.error(format!("error writing to file {}", e));
            return SilkValue::Null;
        }
    }
}

pub fn silk_io_input(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    let mut prompt = String::new();
    if args.len() == 1 {

        let prompt_str = vm.heap_get_string(args[0].clone()).unwrap_or_default();

        prompt = prompt_str.clone();
    }
    else if args.len() != 0 {
        vm.error(String::from("'input' takes 0 or 1 arguments"));
    }
    
    print!("{}", prompt);
    let _ = io::stdout().flush();

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

	let input_val = SilkValue::String(input.trim_end().to_string());
	let handle = vm.heap_allocate(input_val);
	match handle {
		SilkHandle::HeapAllocated(ptr) => {
			return SilkValue::Pointer(ptr);
		}
		_ => unreachable!()
	}
}

pub fn silk_io_append(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'append' expects exactly 2 arguments"));
    }

    let path_str = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    let contents = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    use std::fs::OpenOptions;

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path_str)
    {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{}", contents) {
                vm.error(format!("'append' failed to write to file: {}", e));
                return SilkValue::Null;
            }
            SilkValue::Null
        }
        Err(e) => {
            vm.error(format!("'append' failed to open file: {}", e));
            SilkValue::Null
        }
    }
}

pub fn silk_io_exists(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'exists' expects exactly 1 argument"));
    }

    let path_str = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    SilkValue::Bool(std::path::Path::new(&path_str).exists())
}

pub fn silk_io_delete(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'delete' expects exactly 1 argument"));
    }

    let path_str = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    match std::fs::remove_file(&path_str) {
        Ok(_) => SilkValue::Null,
        Err(e) => {
            vm.error(format!("'delete' failed to delete file: {}", e));
            SilkValue::Null
        }
    }
}

pub fn silk_io_error(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() == 1 {
        let message = vm.heap_get_string(args[0].clone()).unwrap_or_default();
        vm.error(message);
    }
    else {
        vm.error("error() was called".to_string());
    }
    SilkValue::Null
}

pub fn silk_file_construct(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'File' constructor expects 1 argument"));
    }

    
    SilkValue::Null
}

pub fn build_io_map() -> HashMap<String, SilkValue> {
    let mut map = HashMap::new();
    map.insert("print".to_string(), SilkValue::NativeFn(silk_io_print, String::from("print(any, any1, any2...) -> Null; Prints values to the console")));
    map.insert("read".to_string(), SilkValue::NativeFn(silk_io_read, String::from("read(path: String) -> String; Reads a text file into a string")));
    map.insert("write".to_string(), SilkValue::NativeFn(silk_io_write, String::from("write(path: String, content: String) -> Null; Writes a string into a text file")));
    map.insert("append".to_string(), SilkValue::NativeFn(silk_io_append, String::from("append(path: String, content: String) -> Null; Appends a string to a file")));
    map.insert("input".to_string(), SilkValue::NativeFn(silk_io_input, String::from("input() -> String; Reads a line from the console")));
    map.insert("exists".to_string(), SilkValue::NativeFn(silk_io_exists, String::from("exists(path: String) -> Boolean; Checks if a file exists")));
    map.insert("delete".to_string(), SilkValue::NativeFn(silk_io_delete, String::from("delete(path: String) -> Null; Deletes a file")));
    map.insert("error".to_string(), SilkValue::NativeFn(silk_io_error, String::from("error(message: String) -> Null; Causes the program to error out")));
    map.insert("File".to_string(), SilkValue::NativeFn(silk_file_construct, String::from("File(path: String) -> File; Opens a file object from <path>")));
    map
}