use std::time::Instant;

use emu_8080::emulator::State8080;

fn main() {
    let state = State8080::new()
        .loading_file_into_memory_at(
            "/Users/prezi/Developer/emu-8080/resources/invaders.h",
            0x0000,
        )
        .loading_file_into_memory_at(
            "/Users/prezi/Developer/emu-8080/resources/invaders.g",
            0x0800,
        )
        .loading_file_into_memory_at(
            "/Users/prezi/Developer/emu-8080/resources/invaders.f",
            0x1000,
        )
        .loading_file_into_memory_at(
            "/Users/prezi/Developer/emu-8080/resources/invaders.e",
            0x1800,
        );
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
