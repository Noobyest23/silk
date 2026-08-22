use std::ffi::CString;

unsafe extern "C" {
    fn silk_host_register_module_string(module_name: *const std::os::raw::c_char, key: *const std::os::raw::c_char, value: *const std::os::raw::c_char) -> bool;
    fn silk_host_register_module_int(module_name: *const std::os::raw::c_char, key: *const std::os::raw::c_char, value: i32) -> bool;
    fn silk_host_register_module_bool(module_name: *const std::os::raw::c_char, key: *const std::os::raw::c_char, value: i32) -> bool;
    fn silk_host_register_module_native_fn(
        module_name: *const std::os::raw::c_char,
        key: *const std::os::raw::c_char,
        callback: unsafe extern "C" fn(*const std::os::raw::c_char) -> *const std::os::raw::c_char,
        description: *const std::os::raw::c_char,
    ) -> bool;
}

#[unsafe(no_mangle)]
pub extern "C" fn native_greeting(input: *const std::os::raw::c_char) -> *const std::os::raw::c_char {
    let value = unsafe { std::ffi::CStr::from_ptr(input) }
        .to_str()
        .unwrap_or("example");

    let message = CString::new(format!("{} from native plugin", value)).unwrap();
    Box::into_raw(message.into_boxed_c_str()) as *const std::os::raw::c_char
}

#[unsafe(no_mangle)]
pub extern "C" fn silk_load_module() {
    let module_name = CString::new("example_dll").unwrap();
    let hello_key = CString::new("hello").unwrap();
    let hello_value = CString::new("hello from example dll").unwrap();

    unsafe {
        let _ = silk_host_register_module_string(module_name.as_ptr(), hello_key.as_ptr(), hello_value.as_ptr());

        let answer_key = CString::new("answer").unwrap();
        let _ = silk_host_register_module_int(module_name.as_ptr(), answer_key.as_ptr(), 42);

        let ready_key = CString::new("ready").unwrap();
        let _ = silk_host_register_module_bool(module_name.as_ptr(), ready_key.as_ptr(), 1);

        let call_key = CString::new("native_greeting").unwrap();
        let description = CString::new("native_greeting(name: String) -> String").unwrap();
        let _ = silk_host_register_module_native_fn(
            module_name.as_ptr(),
            call_key.as_ptr(),
            native_greeting,
            description.as_ptr(),
        );
    }
}
