use std::time::{Duration, Instant};

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

#[derive(Copy, Clone)]
#[repr(u16)]
enum Interrupt {
    Half = 0,
    End,
}

impl Interrupt {
    fn toggle(&mut self) {
        *self = match self {
            Interrupt::Half => Interrupt::End,
            Interrupt::End => Interrupt::Half,
        }
    }
}

fn run(state: State8080) {
    let mut state = state;

    let mut last_time = None;
    let mut next_interrupt = None;
    let mut which_int = Interrupt::Half;

    loop {
        let now = Instant::now();

        if let None = last_time {
            last_time = Some(now);
            next_interrupt = Some(now + Duration::from_micros(16667));
        }

        if state.interrupt_enabled && now > next_interrupt.unwrap() {
            state = state.generating_interrupt(which_int as u16);
            which_int.toggle();
            next_interrupt = Some(now + Duration::from_micros(8334));
        }

        let since_last = now - last_time.unwrap();

        let cycles_left = 2 * since_last.as_micros();
        let mut cycles_ran = 0;

        while cycles_left > cycles_ran {
            state = state.evaluating_next();
            cycles_ran += state.last_cycles() as u128;
        }

        last_time = Some(now);
    }
}
