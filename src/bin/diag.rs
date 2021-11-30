use emu_8080::emulator::State8080;

fn main() {
    //Fix the stack pointer from 0x6ad to 0x7ad
    // this 0x06 byte 112 in the code, which is
    // byte 112 + 0x100 = 368 in memory
    let sp_correction = vec![0x7];

    let mut state = State8080::new()
        // .loading_buffer_into_memory_at(initial_jmp, 0)
        .loading_file_into_memory_at(
            "/Users/prezi/Developer/emu-8080/resources/cpu_tests/CPUTEST.COM",
            0x0100,
        )
        .setting_memory_at(0xC9, 0x0005);
    // .loading_buffer_into_memory_at(sp_correction, 368);

    state.pc = 0x100;

    println!("Start...");

    run(state);
}

fn run(state: State8080) {
    let mut state = state;

    loop {
        state = state.evaluating_next();

        let addr = state.pc;
        if addr == 5 {
            if state.c == 9 {
                let offset: u16 = state.de().into();
                state.memory[(offset as usize + 3)..]
                    .iter()
                    .take_while(|c| **c != b'$')
                    .map(|c| *c)
                    .for_each(|c| print!("{}", c as char));
                println!();
            } else if state.c == 2 {
                print!("{}", state.e as char);
            }
        }

        if addr == 0 {
            println!("Jumped to 0x0000, halting");
            std::process::exit(0);
        }
    }
}
