use std::{collections::HashMap, fs::{File, OpenOptions}, io::{BufRead, BufReader, Seek, SeekFrom, Write}, sync::{Arc, Mutex}};
use crate::environment::vm::{SilkHandle::{self, HeapAllocated}, VirtualMachine};
use std::io;
use super::super::value::SilkValue;

// @export Modules/IO
/*
    The IO module provides file system, input/output, and console control utilities.
    It includes global IO helper functions as well as an object-oriented File interface for reading and writing streams.
*/

// @export Modules/IO#print
/*
    <b>Signature</b>
    <code>print(any, ...) -> Null</code>

    <p>Prints one or more values to standard output, separated by commas, followed by a newline.</p>

    <b>Parameters:</b>
    - <code>...args</code>: Values to display in the console.

    <b>Returns:</b>
    - <code>Null</code>

    <b>Usage:</b>
    <pre><code># Basic variable definition
var msg = "Hello, world!"
print(msg, 42)</code></pre>
*/
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

// @export Modules/IO#read
/*
    <b>Signature</b>
    <code>read(path: String) -> String</code>

    <p>Reads the entire contents of a file at the given path into a string.</p>

    <b>Parameters:</b>
    - <code>path</code>: The path to the file.

    <b>Returns:</b>
    - <code>String</code>: File contents, or <code>Null</code> on failure.

    <b>Usage:</b>
    <pre><code>var content = read("data.txt")</code></pre>
*/
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

// @export Modules/IO#write
/*
    <b>Signature</b>
    <code>write(path: String, content: String) -> Null</code>

    <p>Writes text content to a target file, creating or completely overwriting it.</p>

    <b>Parameters:</b>
    - <code>path</code>: Destination file path.
    - <code>content</code>: Text payload to write.

    <b>Returns:</b>
    - <code>Null</code>

    <b>Usage:</b>
    <pre><code>write("output.txt", "Hello World")</code></pre>
*/
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

// @export Modules/IO#input
/*
    <b>Signature</b>
    <code>input(prompt: String = "") -> String</code>

    <p>Reads a single line of standard input from the console after displaying an optional prompt string.</p>

    <b>Parameters:</b>
    - <code>prompt</code>: (Optional) Text prompt to show before waiting for user input.

    <b>Returns:</b>
    - <code>String</code>: User input stripped of trailing newlines.

    <b>Usage:</b>
    <pre><code>var name = input("Enter your name: ")</code></pre>
*/
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

// @export Modules/IO#append
/*
    <b>Signature</b>
    <code>append(path: String, content: String) -> Null</code>

    <p>Appends text content to the end of a specified file. Creates the file if it doesn't exist.</p>

    <b>Parameters:</b>
    - <code>path</code>: Destination file path.
    - <code>content</code>: Text payload to append.

    <b>Returns:</b>
    - <code>Null</code>

    <b>Usage:</b>
    <pre><code>append("log.txt", "New entry\n")</code></pre>
*/
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

// @export Modules/IO#exists
/*
    <b>Signature</b>
    <code>exists(path: String) -> Bool</code>

    <p>Checks whether a given path exists on the file system.</p>

    <b>Parameters:</b>
    - <code>path</code>: File or directory path to verify.

    <b>Returns:</b>
    - <code>Bool</code>: <code>true</code> if the path exists, otherwise <code>false</code>.

    <b>Usage:</b>
    <pre><code>if exists("config.json") {
    # File found
}</code></pre>
*/
pub fn silk_io_exists(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'exists' expects exactly 1 argument"));
    }

    let path_str = vm.heap_get_string(args[0].clone()).unwrap_or_default();
    SilkValue::Bool(std::path::Path::new(&path_str).exists())
}

// @export Modules/IO#delete
/*
    <b>Signature</b>
    <code>delete(path: String) -> Null</code>

    <p>Deletes a file from the file system.</p>

    <b>Parameters:</b>
    - <code>path</code>: Path of the file to remove.

    <b>Returns:</b>
    - <code>Null</code>

    <b>Usage:</b>
    <pre><code>delete("temp.tmp")</code></pre>
*/
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

// @export Modules/IO#error
/*
    <b>Signature</b>
    <code>error(message: String = "error() was called") -> Null</code>

    <p>Raises a fatal runtime error with an optional descriptive message, halting VM execution.</p>

    <b>Parameters:</b>
    - <code>message</code>: (Optional) Error reason text.

    <b>Returns:</b>
    - <code>Null</code>

    <b>Usage:</b>
    <pre><code>error("Fatal crash")</code></pre>
*/
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

#[derive(Clone)]
pub struct SilkFileHandle {
    pub path: String,
    pub file: Arc<Mutex<File>>,
}

fn extract_file_handle(vm: &mut VirtualMachine, arg: &SilkValue, fn_name: &str) -> Option<(usize, SilkFileHandle)> {
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

    let Some(SilkValue::NativeData(arc_data)) = map.get("handle") else {
        vm.error(format!("file object missing valid handle in {}", fn_name));
        return None;
    };

    if let Some(file_handle) = arc_data.downcast_ref::<SilkFileHandle>() {
        Some((file_ptr, file_handle.clone()))
    } else {
        vm.error(format!("failed to downcast file handle in {}", fn_name));
        None
    }
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

// @export Modules/IO#File.write
/*
    <b>Signature</b>
    <code>File.write(content: String) -> Null</code>

    <p>Writes content to the handle's target file starting at the current cursor offset.</p>

    <b>Parameters:</b>
    - <code>content</code>: Text buffer to write.

    <b>Returns:</b>
    - <code>Null</code>
*/
pub fn silk_file_write(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'file.write' expects 2 arguments (file, content)"));
        return SilkValue::Null;
    }

    let Some((_, handle)) = extract_file_handle(vm, &args[0], "file.write") else {
        return SilkValue::Null;
    };

    let content = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    if let Ok(mut file) = handle.file.lock() {
        if let Err(e) = file.write_all(content.as_bytes()) {
            vm.error(format!("file.write failed: {}", e));
        }
    }

    SilkValue::Null
}

// @export Modules/IO#File.writeline
/*
    <b>Signature</b>
    <code>File.writeline(line: String) -> Null</code>

    <p>Writes content to the handle's target file and appends a trailing newline character.</p>

    <b>Parameters:</b>
    - <code>line</code>: Line string to write.

    <b>Returns:</b>
    - <code>Null</code>
*/
pub fn silk_file_writeline(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'file.writeline' expects 2 arguments (file, line)"));
        return SilkValue::Null;
    }

    let Some((_, handle)) = extract_file_handle(vm, &args[0], "file.writeline") else {
        return SilkValue::Null;
    };

    let line = vm.heap_get_string(args[1].clone()).unwrap_or_default();

    if let Ok(mut file) = handle.file.lock() {
        let content = format!("{}\n", line);
        if let Err(e) = file.write_all(content.as_bytes()) {
            vm.error(format!("file.writeline failed: {}", e));
        }
    }

    SilkValue::Null
}

// @export Modules/IO#File.cursor
/*
    <b>Signature</b>
    <code>File.cursor(pos: Int) -> Int</code>

    <p>Moves the internal byte read/write stream cursor position within the file.</p>

    <b>Parameters:</b>
    - <code>pos</code>: Absolute byte offset from start of file.

    <b>Returns:</b>
    - <code>Int</code>: New absolute byte cursor position.
*/
pub fn silk_file_cursor(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 2 {
        vm.error(String::from("'file.cursor' expects 2 arguments (file, position)"));
        return SilkValue::Null;
    }

    let Some((_, handle)) = extract_file_handle(vm, &args[0], "file.cursor") else {
        return SilkValue::Null;
    };

    let target_pos = match args[1] {
        SilkValue::Int(i) => i.max(0) as u64,
        _ => {
            vm.error(String::from("'file.cursor' expected an integer for position"));
            return SilkValue::Null;
        }
    };

    if let Ok(mut file) = handle.file.lock() {
        match file.seek(SeekFrom::Start(target_pos)) {
            Ok(new_pos) => return SilkValue::Int(new_pos as i32),
            Err(e) => {
                vm.error(format!("file.cursor seek failed: {}", e));
                return SilkValue::Null;
            }
        }
    }

    SilkValue::Null
}

// @export Modules/IO#File.getline
/*
    <b>Signature</b>
    <code>File.getline() -> String</code>

    <p>Reads and returns the next line of text from the file starting from current cursor position.</p>

    <b>Returns:</b>
    - <code>String</code>: Next text line (excluding newline characters).
*/
pub fn silk_file_getline(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'file.getline' function expects 1 argument"));
        return SilkValue::Null;
    }

    let Some((_, handle)) = extract_file_handle(vm, &args[0], "file.getline") else {
        return SilkValue::Null;
    };

    if let Ok(mut file) = handle.file.lock() {
        let mut reader = BufReader::new(&*file);
        let mut line = String::new();

        match reader.read_line(&mut line) {
            Ok(bytes_read) => {
                // Seek underlying file descriptor to match current buffer offset
                let _ = file.seek(SeekFrom::Current(bytes_read as i64));

                let trimmed = line.trim_end_matches(&['\r', '\n'][..]).to_string();
                let handle = vm.heap_allocate(SilkValue::String(trimmed));
                return match handle {
                    SilkHandle::HeapAllocated(ptr) => SilkValue::Pointer(ptr),
                    _ => unreachable!(),
                };
            }
            Err(e) => {
                vm.error(format!("file.getline failed: {}", e));
                return SilkValue::Null;
            }
        }
    }

    SilkValue::Null
}

// @export Modules/IO#File.getlines
/*
    <b>Signature</b>
    <code>File.getlines() -> List</code>

    <p>Resets the file pointer to byte 0 and reads every line into a list of strings.</p>

    <b>Returns:</b>
    - <code>List</code>: List containing strings for each line in the file.
*/
pub fn silk_file_getlines(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'file.getlines' function expects 1 argument"));
        return SilkValue::Null;
    }

    let Some((_, handle)) = extract_file_handle(vm, &args[0], "file.getlines") else {
        return SilkValue::Null;
    };

    if let Ok(mut file) = handle.file.lock() {
        // Reset cursor to start to read all lines
        if let Err(e) = file.seek(SeekFrom::Start(0)) {
            vm.error(format!("file.getlines failed to seek: {}", e));
            return SilkValue::Null;
        }

        let reader = BufReader::new(&*file);
        let mut elems: Vec<SilkValue> = Vec::new();

        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let handle = vm.heap_allocate(SilkValue::String(l));
                    match handle {
                        SilkHandle::HeapAllocated(ptr) => elems.push(SilkValue::Pointer(ptr)),
                        _ => unreachable!(),
                    }
                }
                Err(e) => {
                    vm.error(format!("file.getlines error reading line: {}", e));
                    return SilkValue::Null;
                }
            }
        }

        let list_handle = vm.heap_allocate(SilkValue::List(elems));
        return match list_handle {
            SilkHandle::HeapAllocated(ptr) => SilkValue::Pointer(ptr),
            _ => unreachable!(),
        };
    }

    SilkValue::Null
}

// @export Modules/IO#File
/*
    <b>Signature</b>
    <code>File(path: String) -> File</code>

    <p>Instantiates an object-oriented stream file reference open for reading and writing.</p>

    <b>Parameters:</b>
    - <code>path</code>: The file system target path.

    <b>Returns:</b>
    - <code>File</code>: An instance of a File object providing streaming methods.

    <b>Usage:</b>
    <pre><code>var f = File("notes.txt")
f.writeline("Hello")
var lines = f.getlines()</code></pre>
*/
pub fn silk_file_construct(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() != 1 {
        vm.error(String::from("'File' constructor expects 1 argument"));
        return SilkValue::Null;
    }

    let path = vm.heap_get_string(args[0].clone()).unwrap_or_default();

    // Open file with Read + Write access, create if it doesn't exist
    let file = match OpenOptions::new().read(true).write(true).create(true).open(&path) {
        Ok(f) => f,
        Err(e) => {
            vm.error(format!("Failed to open file '{}': {}", path, e));
            return SilkValue::Null;
        }
    };

    let handle_struct = SilkFileHandle {
        path: path.clone(),
        file: Arc::new(Mutex::new(file)),
    };

    let mut obj = HashMap::new();
    obj.insert("handle".to_owned(), SilkValue::NativeData(Arc::new(handle_struct)));

    if let HeapAllocated(ptr) = vm.heap_allocate(SilkValue::String(path)) {
        obj.insert("path".to_owned(), SilkValue::Pointer(ptr));
    }

    let methods: [(&str, SilkValue); 5] = [
        ("getline", SilkValue::NativeFn(silk_file_getline, String::from("getline() -> String; Returns the next line"))),
        ("getlines", SilkValue::NativeFn(silk_file_getlines, String::from("getlines() -> List; Returns all lines"))),
        ("write", SilkValue::NativeFn(silk_file_write, String::from("write(content: String); Writes content to file"))),
        ("writeline", SilkValue::NativeFn(silk_file_writeline, String::from("writeline(line: String); Appends line to file"))),
        ("cursor", SilkValue::NativeFn(silk_file_cursor, String::from("cursor(pos: Int) -> Int; Sets byte cursor position"))),
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