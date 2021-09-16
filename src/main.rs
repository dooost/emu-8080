use disassembler::Instruction;
use std::{convert::TryFrom, fs};

mod disassembler;

fn main() {
    let buf = fs::read("/Users/prezi/Developer/emu-8080/resources/invaders.h")
        .expect("Failed to read file");
    println!("Start...");

    let mut output = String::new();

    let mut iter = buf.iter();
    while let Some(byte) = iter.next() {
        let hex_byte = format!("{:#04x}", byte);

        let mut output_line = format!("{}", hex_byte);

        let instruction = Instruction::try_from(*byte);

        match instruction {
            Ok(instruction) => {
                output_line = format!("{}    {}", output_line, instruction.to_string());

                for _i in 1..instruction.size() {
                    let byte = iter.next().expect("Unterminated instruction");
                    let hex_byte = format!("{:#04x}", byte);

                    output_line = format!("{}    {}", output_line, hex_byte);
                }
            }
            Err(()) => (),
        }

        output_line = format!("{}\n", output_line);
        output.push_str(&output_line);
    }

    fs::write("/Users/prezi/Developer/emu-8080/invaders-h.txt", output)
        .expect("Failed to write output file");
}
