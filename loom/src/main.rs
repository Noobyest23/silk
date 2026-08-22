use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use clap::{Parser, Subcommand};
use libloading::{Library, Symbol};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use silk::version;

static HOST_NATIVE_CALLBACK: OnceLock<Mutex<Option<unsafe extern "C" fn(*const std::os::raw::c_char) -> *const std::os::raw::c_char>>> = OnceLock::new();

fn host_native_callback_bridge(
    _vm: &mut silk::VirtualMachine,
    args: &Vec<silk::SilkValue>,
) -> silk::SilkValue {
    let Some(silk::SilkValue::String(value)) = args.first() else {
        return silk::SilkValue::String(String::new());
    };

    let callback_slot = HOST_NATIVE_CALLBACK.get_or_init(|| Mutex::new(None));
    let callback = {
        let lock = callback_slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        *lock
    };

    let Some(callback) = callback else {
        return silk::SilkValue::String(String::new());
    };

    let c_input = CString::new(value.as_str()).unwrap_or_default();
    let result_ptr = unsafe { callback(c_input.as_ptr()) };

    if result_ptr.is_null() {
        return silk::SilkValue::String(String::new());
    }

    let result = unsafe { std::ffi::CStr::from_ptr(result_ptr) };
    match result.to_str() {
        Ok(value) => silk::SilkValue::String(value.to_string()),
        Err(_) => silk::SilkValue::String(String::new()),
    }
}

#[derive(Parser)]
#[command(author, version, about = "Loom CLI for Silk", long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Silk source file
    Run {
        /// Path to Silk source file
        file: PathBuf,
    },
    // Inspect a standard library module
    Inspect {
        module_name: String,
    },
    Session {

    },
    Version {

    },
    #[command(subcommand)]
    Global(GlobalCommands),
}

#[derive(Subcommand)]
enum GlobalCommands {
    /// Read a global variable
    Read {
        name: String,
    },
    /// Write to a global variable
    Write {
        name: String,
        value: String,
    },
    /// Call a global function
    Call {
        expr: Vec<String>,
    },
}

#[unsafe(no_mangle)]
pub extern "C" fn silk_host_register_module_string(module_name: *const std::os::raw::c_char, key: *const std::os::raw::c_char, value: *const std::os::raw::c_char) -> bool {
    if module_name.is_null() || key.is_null() || value.is_null() {
        return false;
    }

    let name = unsafe { std::ffi::CStr::from_ptr(module_name) };
    let key_name = unsafe { std::ffi::CStr::from_ptr(key) };
    let value_name = unsafe { std::ffi::CStr::from_ptr(value) };

    match (name.to_str(), key_name.to_str(), value_name.to_str()) {
        (Ok(module), Ok(key), Ok(value)) => silk::add_module_value(module, key, silk::SilkValue::String(value.to_string())),
        _ => false,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn silk_host_register_module_int(module_name: *const std::os::raw::c_char, key: *const std::os::raw::c_char, value: i32) -> bool {
    if module_name.is_null() || key.is_null() {
        return false;
    }

    let name = unsafe { std::ffi::CStr::from_ptr(module_name) };
    let key_name = unsafe { std::ffi::CStr::from_ptr(key) };

    match (name.to_str(), key_name.to_str()) {
        (Ok(module), Ok(key)) => silk::add_module_value(module, key, silk::SilkValue::Int(value)),
        _ => false,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn silk_host_register_module_bool(module_name: *const std::os::raw::c_char, key: *const std::os::raw::c_char, value: i32) -> bool {
    if module_name.is_null() || key.is_null() {
        return false;
    }

    let name = unsafe { std::ffi::CStr::from_ptr(module_name) };
    let key_name = unsafe { std::ffi::CStr::from_ptr(key) };

    match (name.to_str(), key_name.to_str()) {
        (Ok(module), Ok(key)) => silk::add_module_value(module, key, silk::SilkValue::Bool(value != 0)),
        _ => false,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn silk_host_register_module_native_fn(
    module_name: *const std::os::raw::c_char,
    key: *const std::os::raw::c_char,
    callback: unsafe extern "C" fn(*const std::os::raw::c_char) -> *const std::os::raw::c_char,
    description: *const std::os::raw::c_char,
) -> bool {
    if module_name.is_null() || key.is_null() || description.is_null() {
        return false;
    }

    let name = unsafe { std::ffi::CStr::from_ptr(module_name) };
    let key_name = unsafe { std::ffi::CStr::from_ptr(key) };
    let desc = unsafe { std::ffi::CStr::from_ptr(description) };

    let (module, key_str, desc_str) = match (name.to_str(), key_name.to_str(), desc.to_str()) {
        (Ok(module), Ok(key_str), Ok(desc_str)) => (module, key_str, desc_str),
        _ => return false,
    };

    let callback_slot = HOST_NATIVE_CALLBACK.get_or_init(|| Mutex::new(None));
    {
        let mut lock = callback_slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        *lock = Some(callback);
    }

    silk::register_module_native_fn(module, key_str, host_native_callback_bridge, desc_str)
}

fn main() {
    let cli = Cli::parse();

    unsafe {
        silk::init();
    }

    match cli.command {
        Commands::Run { file } => {
            execute_run(file);
        }
        Commands::Inspect { module_name } => {
            inspect_module(module_name);
        }
        Commands::Session {} => {  
            run_session();
        }
        Commands::Global(global_cmd) => {
            match global_cmd {
                GlobalCommands::Read { name } => {
                    execute_read_global(name);
                }
                GlobalCommands::Write { name, value } => {
                    execute_write_global(name, value);
                }
                GlobalCommands::Call { expr } => {
                    execute_global_call(expr);
                }
            }
        }
        Commands::Version {  } => {
            unsafe {
                println!("{}", version())
            }
        }
    }
}

/// Extracted runner helper function so it can be called from both Main and Session
fn execute_run(file: PathBuf) {
    let path_str = file.to_string_lossy().to_string();
    let c_path = CString::new(path_str).expect("CString::new failed");
    unsafe {
        silk::run(c_path.as_ptr());
    }
}

fn inspect_module(module: String) {
    let mod_str = CString::new(module).expect("CString new failed");
    unsafe {
        silk::inspect(mod_str.as_ptr());
    }
}

/// The interactive session state loop
fn run_session() {
    let mut rl = DefaultEditor::new().expect("Failed to initialize session editor");
    println!("Loom Session Mode. Type 'help' to see available commands, or 'exit' to end.");

    loop {
        let readline = rl.readline("session > ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                
                let _ = rl.add_history_entry(trimmed);

                // Use a helper to split arguments while respecting quotes
                let args = match tokenize_input(trimmed) {
                    Ok(a) => a,
                    Err(e) => {
                        println!("Parsing Error: {}", e);
                        continue;
                    }
                };

                if args.is_empty() {
                    continue;
                }

                // Command Dispatcher
                match args[0].as_str() {
                    "exit" | "quit" => {
                        println!("Goodbye!");
                        break;
                    }
                    "help" => {
                        print_session_help();
                    }
                    "run" => {
                        if args.len() < 2 {
                            println!("Error: 'run' command requires a file path target.");
                        } else {
                            execute_run(PathBuf::from(&args[1]));
                        }
                    }
                    "inspect" => {
                        if args.len() < 2 {
                            println!("Error: 'inspect' command requires a module name.");
                        } else {
                            inspect_module(args[1].clone());
                        }
                    }
                    "plugin" => {
                        if args.len() < 2 {
                            println!("Error: 'plugin' command requires a path to a shared library.");
                        } else {
                            match load_plugin(&args[1]) {
                                Ok(_) => println!("Plugin loaded: {}", args[1]),
                                Err(err) => println!("{}", err),
                            }
                        }
                    }
                    "global" => {
                        if args.len() < 2 {
                            println!("Error: 'global' command requires an operation. Usage: global read <name>, global write <name> <value>, or global call <function>(...) ");
                        } else {
                            match args[1].as_str() {
                                "read" => {
                                    if args.len() < 3 {
                                        println!("Error: 'global read' requires a variable name.");
                                    } else {
                                        execute_read_global(args[2].clone());
                                    }
                                }
                                "write" => {
                                    if args.len() < 4 {
                                        println!("Error: 'global write' requires a name and a value. Usage: global write <name> <value>");
                                    } else {
                                        execute_write_global(args[2].clone(), args[3].clone());
                                    }
                                }
                                "call" => {
                                    if args.len() < 3 {
                                        println!("Error: 'global call' requires a function call expression. Example: global call my_func(1, 2, 3)");
                                    } else {
                                        execute_global_call(args[2..].to_vec());
                                    }
                                }
                                _ => {
                                    println!("Unknown global operation: '{}'. Use 'read', 'write', or 'call'.", args[1]);
                                }
                            }
                        }
                    }
                    unknown => {
                        println!("Unknown session command: '{}'. Type 'help' for options.", unknown);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C received. Exiting session.");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D received. Exiting session.");
                break;
            }
            Err(err) => {
                println!("Error reading input: {:?}", err);
                break;
            }
        }
    }
}

/// Helper function to print inline help menu
fn print_session_help() {
    println!("\nAvailable Session Commands:");
    println!("  run <file>                   Execute a Silk source file (supports spaces inside quotes)");
    println!("  inspect <module>             Inspect a standard library module");
    println!("  plugin <path>               Load a shared library and call silk_load_module()");
    println!("  global read <name>                Read a global variable");
    println!("  global write <name> <value>       Set a global variable");
    println!("  global call <function>(...)       Call a globally registered function");
    println!("  help                              Show this help text");
    println!("  exit / quit                       Terminate the active session\n");
}

fn resolve_plugin_path(path: &str) -> Result<PathBuf, String> {
    let raw = Path::new(path);

    if raw.is_absolute() || raw.has_root() || raw.extension().is_some() {
        let candidate = raw.to_path_buf();
        if candidate.exists() {
            return Ok(candidate);
        }
        return Err(format!("Plugin path not found: {}", path));
    }

    let candidates = [
        raw.to_path_buf(),
        PathBuf::from(format!("{}.so", path)),
        PathBuf::from(format!("{}.dll", path)),
        PathBuf::from(format!("{}.dylib", path)),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(format!("Plugin path not found: {}", path))
}

fn load_plugin(path: &str) -> Result<(), String> {
    let resolved = resolve_plugin_path(path)?;
    let library = unsafe {
        Library::new(&resolved)
            .map_err(|e| format!("Failed to load plugin '{}': {}", resolved.display(), e))?
    };

    unsafe {
        let func: Symbol<unsafe extern "C" fn()> = library
            .get(b"silk_load_module")
            .map_err(|e| format!("Plugin '{}' does not export silk_load_module(): {}", resolved.display(), e))?;
        func();
    }

    std::mem::forget(library);
    Ok(())
}

/// Tokenizes input into arguments, keeping quoted substrings together.
/// Allows paths like: run "my projects/script.silk"
fn tokenize_input(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    for c in input.chars() {
        match c {
            '"' | '\'' => {
                if in_quotes {
                    current.push(c);
                    if c == quote_char {
                        in_quotes = false; // Closed matching quote
                    }
                } else {
                    in_quotes = true;
                    quote_char = c;
                    current.push(c);
                }
            }
            ' ' | '\t' => {
                if in_quotes {
                    current.push(c);
                } else if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if in_quotes {
        return Err("Unmatched quote found in command input.".to_string());
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

fn execute_global_call(expr: Vec<String>) {
    let call_expr = expr.join(" ");
    println!("Executing global call: {}", call_expr);
    silk::run_source(&call_expr);
}

fn execute_read_global(name: String) {
    let c_name = CString::new(name.clone()).expect("CString::new failed for global name");
    const BUFFER_SIZE: usize = 4096;
    let mut buffer = vec![0u8; BUFFER_SIZE];

    unsafe {
        // Try reading as string first (largest buffer available)
        if silk::get_global_string(c_name.as_ptr(), buffer.as_mut_ptr() as *mut std::os::raw::c_char, BUFFER_SIZE) {
            if let Ok(s) = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const std::os::raw::c_char).to_str() {
                println!("{} = \"{}\" (string)", name, s);
            }
            return;
        }

        // Try reading as int
        let int_val = silk::get_global_int(c_name.as_ptr());
        if int_val != 0 {
            println!("{} = {} (int)", name, int_val);
            return;
        }

        // Try reading as float
        let float_val = silk::get_global_float(c_name.as_ptr());
        if float_val != 0.0 {
            println!("{} = {} (float)", name, float_val);
            return;
        }

        // Try reading as bool
        let bool_val = silk::get_global_bool(c_name.as_ptr());
        println!("{} = {} (bool/int)", name, bool_val);
    }
}

fn execute_write_global(name: String, value: String) {
    let c_name = CString::new(name).expect("CString::new failed for global name");

    if value.eq_ignore_ascii_case("true") {
        unsafe { silk::set_global_bool(c_name.as_ptr(), 1); }
        println!("Global '{}' set to true", c_name.to_string_lossy());
        return;
    } else if value.eq_ignore_ascii_case("false") {
        unsafe { silk::set_global_bool(c_name.as_ptr(), 0); }
        println!("Global '{}' set to false", c_name.to_string_lossy());
        return;
    }

    if let Ok(parsed_int) = value.parse::<std::os::raw::c_int>() {
        unsafe { silk::set_global_int(c_name.as_ptr(), parsed_int); }
        println!("Global '{}' set to {} (int)", c_name.to_string_lossy(), parsed_int);
        return;
    }

    if let Ok(parsed_float) = value.parse::<std::os::raw::c_float>() {
        unsafe { silk::set_global_float(c_name.as_ptr(), parsed_float); }
        println!("Global '{}' set to {} (float)", c_name.to_string_lossy(), parsed_float);
        return;
    }

    let c_val = CString::new(value.clone()).expect("CString::new failed for global value");
    unsafe {
        silk::set_global_string(c_name.as_ptr(), c_val.as_ptr());
    }
    println!("Global '{}' set to \"{}\" (string)", c_name.to_string_lossy(), value);
}