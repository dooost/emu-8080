use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::null_mut;

use crate::emulator::{DummyIOHandler, State8080};

fn create_raw_pointer(state: State8080) -> *mut State8080 {
    Box::into_raw(Box::new(state))
}

#[no_mangle]
pub extern "C" fn state8080_new() -> *mut State8080 {
    create_raw_pointer(State8080::new())
}

#[no_mangle]
pub extern "C" fn state8080_free(ptr: *mut State8080) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn state8080_evaluating_next(ptr: *mut State8080) -> *mut State8080 {
    let state = unsafe { Box::from_raw(ptr) };

    create_raw_pointer(state.evaluating_next::<DummyIOHandler>(None))
}

#[no_mangle]
pub extern "C" fn state8080_loading_file_into_memory_at(
    ptr: *mut State8080,
    path: *const c_char,
    index: u16,
) -> *mut State8080 {
    let cstr_path = unsafe { CStr::from_ptr(path) };
    let path = match cstr_path.to_str() {
        Err(_) => return null_mut(),
        Ok(string) => string,
    };

    let state = unsafe { Box::from_raw(ptr) };

    create_raw_pointer(state.loading_file_into_memory_at(path, index))
}
