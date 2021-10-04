use std::time::Instant;

use emu_8080::emulator::State8080;

fn main() {
    //JMP 0x100 - insert at 0
    let initial_jmp = vec![0xc3, 0, 0x01];

    //Fix the stack pointer from 0x6ad to 0x7ad
    // this 0x06 byte 112 in the code, which is
    // byte 112 + 0x100 = 368 in memory
    let sp_correction = vec![0x7];

    //Skip DAA test - insert at 0x59c
    let skip_daa = vec![0xc3, 0xc2, 0x05];

    let state = State8080::new()
        .loading_buffer_into_memory_at(initial_jmp, 0)
        .loading_file_into_memory_at(
            "/Users/prezi/Developer/emu-8080/resources/cpudiag.bin",
            0x0100,
        )
        .loading_buffer_into_memory_at(sp_correction, 368)
        .loading_buffer_into_memory_at(skip_daa, 0x59c);

    println!("Start...");

    run(state);
}

fn run(state: State8080) {
    let mut state = state;

    loop {
        state = state.evaluating_next();
    }
}
