use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{Read, Write};
use std::os::raw::{c_char, c_float, c_int};
use std::ptr;
use std::sync::Mutex;

use lazy_static::lazy_static;

lazy_static! {

    // Global file handle table
    static ref FILE_HANDLES: Mutex<HashMap<i32, File>> = {
        Mutex::new(HashMap::new())
    };

    // Global allocation table for malloc/free tracking
    static ref ALLOCATIONS: Mutex<HashMap<usize, AllocationInfo>> = {
        Mutex::new(HashMap::new())
    };
}

#[derive(Debug, Clone)]
struct AllocationInfo {
    size: usize,
    is_valid: bool,
}

static mut NEXT_HANDLE: i32 = 1;

fn get_handles() -> &'static Mutex<HashMap<i32, File>> {
    &FILE_HANDLES
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
            unsafe {
                NEXT_HANDLE += 1;
            }
            map.insert(handle, f);
            handle
        }
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
        match file.read(&mut buffer) {
            Ok(n) => {
                buffer.truncate(n);
                match CString::new(buffer) {
                    Ok(c_string) => c_string.into_raw(),
                    Err(_) => ptr::null(),
                }
            }
            Err(_) => ptr::null(),
        }
    } else {
        ptr::null()
    }
}

#[no_mangle]
pub extern "C" fn cscript_system(command: *const c_char) -> c_int {
    if command.is_null() {
        return -1;
    }

    let command_str = unsafe { CStr::from_ptr(command).to_string_lossy() };

    // Use sh -c to execute the command string
    match std::process::Command::new("sh")
        .arg("-c")
        .arg(command_str.as_ref())
        .status()
    {
        Ok(status) => status.code().unwrap_or(-1),
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn cscript_getenv(name: *const c_char) -> *const c_char {
    if name.is_null() {
        return ptr::null();
    }

    let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };

    match std::env::var(name_str.as_ref()) {
        Ok(val) => match CString::new(val) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => ptr::null(),
        },
        Err(_) => ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn cscript_malloc(size: c_int) -> *mut c_char {
    if size <= 0 {
        eprintln!("malloc error: size must be positive (got {})", size);
        return ptr::null_mut();
    }

    // Allocate memory as a boxed slice
    let layout = std::alloc::Layout::from_size_align(size as usize, 8).unwrap();
    let ptr = unsafe { std::alloc::alloc(layout) as *mut c_char };

    if ptr.is_null() {
        eprintln!("malloc error: allocation failed for size {}", size);
        return ptr::null_mut();
    }

    // Track the allocation
    let allocations = &ALLOCATIONS;
    let mut map = allocations.lock().unwrap();
    map.insert(
        ptr as usize,
        AllocationInfo {
            size: size as usize,
            is_valid: true,
        },
    );

    ptr
}

#[no_mangle]
pub extern "C" fn cscript_free(ptr: *mut c_char) -> c_int {
    if ptr.is_null() {
        eprintln!("free error: attempting to free null pointer");
        return -1;
    }

    let allocations = &ALLOCATIONS;
    let mut map = allocations.lock().unwrap();

    let addr = ptr as usize;

    // Check if pointer exists in allocation table
    if let Some(info) = map.get_mut(&addr) {
        // Check for double-free
        if !info.is_valid {
            eprintln!("free error: double free detected at address 0x{:x}", addr);
            return -1;
        }

        // Mark as freed (for use-after-free detection)
        info.is_valid = false;

        // Deallocate the memory
        let layout = std::alloc::Layout::from_size_align(info.size, 8).unwrap();
        unsafe {
            std::alloc::dealloc(ptr as *mut u8, layout);
        }

        // Keep in tracking table but marked invalid to detect double-free/use-after-free
        // map.remove(&addr);

        0
    } else {
        eprintln!(
            "free error: pointer 0x{:x} was not allocated by malloc",
            addr
        );
        -1
    }
}

#[no_mangle]
pub extern "C" fn cscript_check_bounds(ptr: *mut c_char, offset: c_int) -> c_int {
    if ptr.is_null() {
        eprintln!("bounds check error: null pointer access");
        return 0;
    }

    let allocations = &ALLOCATIONS;
    let map = allocations.lock().unwrap();

    let addr = ptr as usize;

    if let Some(info) = map.get(&addr) {
        if !info.is_valid {
            eprintln!(
                "bounds check error: use-after-free detected at address 0x{:x}",
                addr
            );
            return 0;
        }

        if offset < 0 || (offset as usize) >= info.size {
            eprintln!(
                "bounds check error: offset {} out of bounds for allocation of size {} at 0x{:x}",
                offset, info.size, addr
            );
            return 0;
        }

        1
    } else {
        eprintln!(
            "bounds check error: pointer 0x{:x} was not allocated by malloc",
            addr
        );
        0
    }
}
