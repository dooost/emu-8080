use std::{convert::TryFrom, fs};

use disassembler::Instruction;
use emulator::State8080;

mod disassembler;
mod emulator;

fn main() {
    let mut buf = vec![];

    let mut buf_h = fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.h")
        .expect("Failed to read file");
    let mut buf_g = fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.g")
        .expect("Failed to read file");
    let mut buf_f = fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.f")
        .expect("Failed to read file");
    let mut buf_e = fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.e")
        .expect("Failed to read file");

    // let state = State8080::new()
    //     .loading_buffer_into_memory_at(buf_h, 0)
    //     .loading_buffer_into_memory_at(buf_g, 0x800)
    //     .loading_buffer_into_memory_at(buf_f, 0x1000)
    //     .loading_buffer_into_memory_at(buf_e, 0x1800);

    // println!("A");
    buf.append(&mut buf_h);
    buf.append(&mut buf_g);
    buf.append(&mut buf_f);
    buf.append(&mut buf_e);

    println!("Start...");

    let mut output = String::new();

    let mut iter = buf.iter();
    let mut counter: usize = 0;
    while let Some(byte) = iter.next() {
        let hex_counter = format!("{:04x}", counter);
        let hex_byte = format!("{:#04x}", byte);

        let mut output_line = format!("{}    {}", hex_counter, hex_byte);

        counter += 1;

        let instruction = Instruction::try_from(*byte);
        match instruction {
            Ok(instruction) => {
                output_line = format!("{}    {}", output_line, instruction.to_string());

                let mut next_bytes = vec![];
                for _i in 1..instruction.size() {
                    let byte = iter.next().expect("Unterminated instruction");
                    next_bytes.push(*byte);
                    counter += 1;
                }

                let mut next_bytes_iter = next_bytes.iter();
                if let Some(next) = next_bytes_iter.next() {
                    let mut adr_str = format!("{:02x}", next);

                    if let Some(next) = next_bytes_iter.next() {
                        adr_str = format!("${:02x}{}", next, adr_str);
                    } else {
                        adr_str = format!("#${}", adr_str);
                    }

                    output_line = format!("{}    {}", output_line, adr_str);
                }
            }
            Err(()) => (),
        }

        output_line = format!("{}\n", output_line);
        output.push_str(&output_line);
    }

    fs::write("/Users/prezi/Developer/emu-8080/invaders.txt", output)
        .expect("Failed to write output file");
}
