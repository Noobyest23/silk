use std::{collections::HashMap, fs::File, io::Write};
use crate::environment::vm::{SilkHandle::{self, HeapAllocated}, VirtualMachine};
use std::io;
use super::super::value::SilkValue;


pub fn silk_io_print(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
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
            if let Err(e) = write!(file, "{}", contents) {
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

fn extract_file_info(vm: &mut VirtualMachine, arg: &SilkValue, fn_name: &str) -> Option<(usize, HashMap<String, SilkValue>, String)> {
    let file_ptr = match arg {
        SilkValue::Pointer(p) => *p,
        _ => {
            vm.error(format!("{} expects a File object as the first argument", fn_name));
            return None;
        }
    };

    let Some(SilkValue::Object(map)) = vm.heap.get(&file_ptr).cloned() else {
        vm.error(format!("{} expected a heap object", fn_name));
        return None;
    };

    let path = match map.get("path") {
        Some(SilkValue::Pointer(pp)) => vm.heap_get_string(SilkValue::Pointer(*pp)).unwrap_or_default(),
        Some(SilkValue::String(s)) => s.clone(),
        _ => {
            vm.error(format!("file object missing 'path' in {}", fn_name));
            return None;
        }
    };

    Some((file_ptr, map, path))
}

pub fn silk_file_write(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'file.write' expects 2 arguments (file, content)"));
        return SilkValue::Null;
    }

    let Some((file_ptr, map, path)) = extract_file_info(vm, &args[0], "file.write") else {
        return SilkValue::Null;
    };

    let content = vm.heap_get_string(args[1].clone()).unwrap_or_default();
    let new_lines: Vec<String> = content.lines().map(String::from).collect();

    if !new_lines.is_empty() {
        write_lines_at_pos(vm, file_ptr, map, &path, new_lines);
    }

    SilkValue::Null
}

pub fn silk_file_writeline(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'file.writeline' expects 2 arguments (file, line)"));
        return SilkValue::Null;
    }

    let Some((file_ptr, map, path)) = extract_file_info(vm, &args[0], "file.writeline") else {
        return SilkValue::Null;
    };

    let line = vm.heap_get_string(args[1].clone()).unwrap_or_default();
    write_lines_at_pos(vm, file_ptr, map, &path, vec![line]);

    SilkValue::Null
}

// Helper function to insert/overwrite lines at `pos` and advance the cursor
fn write_lines_at_pos(
    vm: &mut VirtualMachine,
    file_ptr: usize,
    mut map: HashMap<String, SilkValue>,
    path: &str,
    new_lines: Vec<String>,
) {
    let pos = match map.get("pos") {
        Some(SilkValue::Int(i)) => (*i).max(0) as usize,
        _ => 0usize,
    };

    // Load existing lines from disk if file exists
    let existing_content = std::fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<String> = if existing_content.is_empty() {
        Vec::new()
    } else {
        existing_content.lines().map(String::from).collect()
    };

    // Pad file with empty lines if `pos` exceeds current length
    while lines.len() < pos {
        lines.push(String::new());
    }

    // Insert or replace lines starting at `pos`
    for (offset, line) in new_lines.iter().enumerate() {
        let idx = pos + offset;
        if idx < lines.len() {
            lines[idx] = line.clone();
        } else {
            lines.push(line.clone());
        }
    }

    // Update `pos` cursor on the file object
    let new_pos = pos + new_lines.len();
    map.insert("pos".to_string(), SilkValue::Int(new_pos as i32));
    vm.heap.insert(file_ptr, SilkValue::Object(map));

    // Save formatted content back to disk
    let mut final_content = lines.join("\n");
    if !final_content.is_empty() {
        final_content.push('\n');
    }

    if let Err(e) = std::fs::write(path, final_content) {
        vm.error(format!("Failed to write to file: {}", e));
    }
}

pub fn silk_file_cursor(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'file.cursor' expects 2 arguments (file, position)"));
        return SilkValue::Null;
    }

    let Some((file_ptr, mut map, path)) = extract_file_info(vm, &args[0], "file.cursor") else {
        return SilkValue::Null;
    };

    let target_pos = match args[1] {
        SilkValue::Int(i) => i,
        _ => {
            vm.error(String::from("'file.cursor' expected an integer for position"));
            return SilkValue::Null;
        }
    };

    // Calculate maximum position (total lines) in file
    let max_pos = match std::fs::read_to_string(&path) {
        Ok(contents) => contents.lines().count() as i32,
        Err(_) => 0,
    };

    // Clamp value between 0 and max_pos
    let clamped_pos = target_pos.clamp(0, max_pos);

    map.insert("pos".to_string(), SilkValue::Int(clamped_pos));
    vm.heap.insert(file_ptr, SilkValue::Object(map));

    SilkValue::Int(clamped_pos)
}

pub fn silk_file_getline(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'file.getline' function expects 1 argument"));
        return SilkValue::Null;
    }

    let Some((file_ptr, mut map, path)) = extract_file_info(vm, &args[0], "file.getline") else {
        return SilkValue::Null;
    };

    let pos = match map.get("pos") {
        Some(SilkValue::Int(i)) => (*i).max(0) as usize,
        _ => 0usize,
    };

    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            let total_chars = contents.chars().count();

            // If pos is at or past the end of the file, return ""
            if pos >= total_chars {
                let handle = vm.heap_allocate(SilkValue::String(String::new()));
                return match handle {
                    SilkHandle::HeapAllocated(ptr) => SilkValue::Pointer(ptr),
                    _ => unreachable!(),
                };
            }

            // Slice starting from character index `pos` without collecting into intermediate Strings
            let remaining_slice: String = contents.chars().skip(pos).collect();
            let line_content = remaining_slice.split('\n').next().unwrap_or("");

            // Determine advancement offset
            let chars_read = line_content.chars().count();
            let has_newline = remaining_slice.chars().nth(chars_read) == Some('\n');
            let new_pos = pos + chars_read + if has_newline { 1 } else { 0 };

            // Update file cursor state
            map.insert("pos".to_string(), SilkValue::Int(new_pos as i32));
            vm.heap.insert(file_ptr, SilkValue::Object(map));

            // Strip trailing carriage returns for cross-platform compliance (\r\n)
            let trimmed = line_content.strip_suffix('\r').unwrap_or(line_content);

            let handle = vm.heap_allocate(SilkValue::String(trimmed.to_string()));
            match handle {
                SilkHandle::HeapAllocated(ptr) => SilkValue::Pointer(ptr),
                _ => unreachable!(),
            }
        }
        Err(e) => {
            vm.error(format!("file.getline failed to read file: {}", e));
            SilkValue::Null
        }
    }
}

pub fn silk_file_getlines(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'file.getlines' function expects 1 argument"));
        return SilkValue::Null;
    }

    let Some((_, _, path)) = extract_file_info(vm, &args[0], "file.getlines") else {
        return SilkValue::Null;
    };

    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            let lines: Vec<&str> = contents.lines().collect();
            let mut elems: Vec<SilkValue> = Vec::with_capacity(lines.len());
            for l in lines {
                let handle = vm.heap_allocate(SilkValue::String(l.to_string()));
                match handle {
                    SilkHandle::HeapAllocated(ptr) => elems.push(SilkValue::Pointer(ptr)),
                    _ => unreachable!(),
                }
            }

            let list_handle = vm.heap_allocate(SilkValue::List(elems));
            match list_handle {
                SilkHandle::HeapAllocated(ptr) => SilkValue::Pointer(ptr),
                _ => unreachable!(),
            }
        }
        Err(e) => {
            vm.error(format!("file.getlines failed to read file: {}", e));
            SilkValue::Null
        }
    }
}

pub fn silk_file_construct(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'File' constructor expects 1 argument"));
        return SilkValue::Null;
    }

    let path = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    let mut obj = HashMap::new();
    obj.insert("pos".to_owned(), SilkValue::Int(0));

    // Attach path
    if let HeapAllocated(ptr) = vm.heap_allocate(SilkValue::String(path)) {
        obj.insert("path".to_owned(), SilkValue::Pointer(ptr));
    }

    // Attach method closures
    let methods: [(&str, SilkValue); 5] = [
        ("getline", SilkValue::NativeFn(silk_file_getline, String::from("getline() -> String; Returns the next line"))),
        ("getlines", SilkValue::NativeFn(silk_file_getlines, String::from("getlines() -> List; Returns all lines"))),
        ("write", SilkValue::NativeFn(silk_file_write, String::from("write(content: String); Writes content to file"))),
        ("writeline", SilkValue::NativeFn(silk_file_writeline, String::from("writeline(line: String); Appends line to file"))),
        ("cursor", SilkValue::NativeFn(silk_file_cursor, String::from("cursor(pos: Int) -> Int; Sets line cursor position"))),
    ];

    for (name, native_fn) in methods {
        if let HeapAllocated(ptr) = vm.heap_allocate(native_fn) {
            obj.insert(name.to_string(), SilkValue::Pointer(ptr));
        }
    }

    let obj_handle = vm.heap_allocate(SilkValue::Object(obj));
    match obj_handle {
        HeapAllocated(ptr) => SilkValue::Pointer(ptr),
        _ => unreachable!(),
    }
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