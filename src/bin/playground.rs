use emu_8080::emulator::State8080;

fn main() {
    let mem: Vec<u8> = vec![0xDB, 0x00, 0xD3, 0x01, 0xDB, 0x02, 0xD3, 0x03];
    let state = State8080::new()
        .loading_buffer_into_memory_at(mem, 0)
        .setting_in_handler(|state, b| {
            println!("In: {}", b);
            state
        })
        .setting_out_handler(|state, b| {
            println!("Out: {}", b);
            state
        });

    println!("Start...");

    run(state);
}

fn run(state: State8080) {
    let mut state = state;

    loop {
        state = state.evaluating_next();
    }
}
