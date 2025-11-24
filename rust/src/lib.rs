use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void, c_float};
use std::fs::File;
use std::io::{Read, Write};
use std::ptr;
use std::sync::Mutex;
use std::collections::HashMap;

// Global file handle table
static mut FILE_HANDLES: Option<Mutex<HashMap<i32, File>>> = None;
static mut NEXT_HANDLE: i32 = 1;

fn get_handles() -> &'static Mutex<HashMap<i32, File>> {
    unsafe {
        if FILE_HANDLES.is_none() {
            FILE_HANDLES = Some(Mutex::new(HashMap::new()));
        }
        FILE_HANDLES.as_ref().unwrap()
    }
}

#[no_mangle]
pub extern "C" fn cscript_print_int(val: c_int) {
    println!("{}", val);
}

#[no_mangle]
pub extern "C" fn cscript_print_float(val: c_float) {
    println!("{}", val);
}

#[no_mangle]
pub extern "C" fn cscript_print_string(val: *const c_char) {
    if val.is_null() {
        println!("(null)");
        return;
    }
    let c_str = unsafe { CStr::from_ptr(val) };
    if let Ok(s) = c_str.to_str() {
        println!("{}", s);
    } else {
        println!("(invalid utf-8)");
    }
}

#[no_mangle]
pub extern "C" fn cscript_fopen(filename: *const c_char, mode: *const c_char) -> c_int {
    if filename.is_null() || mode.is_null() {
        return -1;
    }
    
    let filename_str = unsafe { CStr::from_ptr(filename).to_string_lossy() };
    let mode_str = unsafe { CStr::from_ptr(mode).to_string_lossy() };
    
    let file = if mode_str == "w" {
        File::create(filename_str.as_ref())
    } else if mode_str == "r" {
        File::open(filename_str.as_ref())
    } else {
        return -1;
    };
    
    match file {
        Ok(f) => {
            let handles = get_handles();
            let mut map = handles.lock().unwrap();
            let handle = unsafe { NEXT_HANDLE };
            unsafe { NEXT_HANDLE += 1; }
            map.insert(handle, f);
            handle
        },
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cscript_fwrite(handle: c_int, data: *const c_char) -> c_int {
    if data.is_null() {
        return 0;
    }
    
    let handles = get_handles();
    let mut map = handles.lock().unwrap();
    
    if let Some(file) = map.get_mut(&handle) {
        let data_slice = unsafe { CStr::from_ptr(data).to_bytes() };
        match file.write_all(data_slice) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn cscript_fclose(handle: c_int) -> c_int {
    let handles = get_handles();
    let mut map = handles.lock().unwrap();
    
    if map.remove(&handle).is_some() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn cscript_fread(handle: c_int, size: c_int) -> *const c_char {
    if size <= 0 {
        return ptr::null();
    }
    
    let handles = get_handles();
    let mut map = handles.lock().unwrap();
    
    if let Some(file) = map.get_mut(&handle) {
        let mut buffer = vec![0u8; size as usize];
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                match CString::new(buffer) {
                    Ok(c_string) => c_string.into_raw(),
                    Err(_) => ptr::null(),
                }
            },
            Err(_) => ptr::null(),
        }
    } else {
        ptr::null()
    }
}
