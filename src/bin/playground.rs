use emu_8080::emulator::{IOHandler, State8080};

struct CoolIOHandler;

impl IOHandler for CoolIOHandler {
    fn inp(&mut self, state: State8080, v: u8) -> State8080 {
        println!("In {}", v);

        state
    }

    fn out(&mut self, state: State8080, v: u8) -> State8080 {
        println!("Out {}", v);

        state
    }
}

fn main() {
    let mem: Vec<u8> = vec![0xDB, 0x00, 0xD3, 0x01, 0xDB, 0x02, 0xD3, 0x03];
    let state = State8080::new().loading_buffer_into_memory_at(mem, 0);

    println!("Start...");

    run(state);
}

fn run(state: State8080) {
    let mut io_handler = CoolIOHandler;
    let mut state = state;

    loop {
        state = state.evaluating_next(Some(&mut io_handler));
    }
}
