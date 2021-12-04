use std::path::Path;

use emu_8080::emulator::State8080;

#[allow(dead_code)]
pub fn run_suite(path: impl AsRef<Path>) {
    let mut state = create_state_with_rom(path.as_ref());

    let filename = path
        .as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    println!("Starting running suite {}...", filename);

    loop {
        state = state.evaluating_next();

        print_output(&state);

        if state.pc == 0 {
            println!();
            println!("Jumped to 0x0000, halting");
            break;
        }
    }
}

pub fn create_state_with_rom(path: impl AsRef<Path>) -> State8080 {
    let mut state = State8080::new()
        .loading_file_into_memory_at(path.as_ref(), 0x0100)
        .setting_memory_at(0xC9, 0x0005);
    state.pc = 0x100;

    state
}

pub fn print_output(state: &State8080) {
    let addr = state.pc;
    if addr == 5 {
        if state.c == 9 {
            let offset: u16 = state.de().into();
            state.memory[(offset as usize)..]
                .iter()
                .take_while(|c| **c != b'$')
                .map(|c| *c)
                .for_each(|c| print!("{}", c as char));
            println!();
        } else if state.c == 2 {
            print!("{}", state.e as char);
        }
    }
}
