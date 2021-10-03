use crate::emulator::State8080;

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

    create_raw_pointer(state.evaluating_next())
}
