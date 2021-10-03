use std::time::Instant;

use emulator::State8080;

mod disassembler;
mod emulator;

fn main() {
    let buf_h = std::fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.h")
        .expect("Failed to read file");
    let buf_g = std::fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.g")
        .expect("Failed to read file");
    let buf_f = std::fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.f")
        .expect("Failed to read file");
    let buf_e = std::fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.e")
        .expect("Failed to read file");

    let state = State8080::new()
        .loading_buffer_into_memory_at(buf_h, 0)
        .loading_buffer_into_memory_at(buf_g, 0x800)
        .loading_buffer_into_memory_at(buf_f, 0x1000)
        .loading_buffer_into_memory_at(buf_e, 0x1800);
    // .setting_in_handler(|byte| println!("{}", byte))
    // .setting_out_handler(|byte| println!("{}", byte));

    println!("Start...");

    run(state);
}

fn run(state: State8080) {
    let mut state = state;

    let mut last_interrupt = Instant::now();

    loop {
        state = state.evaluating_next();

        if Instant::now().duration_since(last_interrupt).as_secs_f32() > (1.0 / 60.0)
            && state.interrupt_enabled
        {
            state = state.generating_interrupt(2);
            last_interrupt = Instant::now();
        }
    }
}
