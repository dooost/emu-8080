use std::convert::TryFrom;

use bitflags::bitflags;

use crate::disassembler::Instruction;

bitflags! {
    pub struct ConditionCodes: u8 {
        const Z = 0b00000001;
        const S = 0b00000010;
        const P = 0b00000100;
        const CY = 0b00001000;
        const AC = 0b00010000;
        const PAD = 0b11100000;
    }
}

#[derive(Default)]
pub struct State8080 {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub memory: Vec<u8>,
    pub cc: ConditionCodes,
    pub int_enable: u8,
}

impl Default for ConditionCodes {
    fn default() -> Self {
        Self { bits: 0 }
    }
}

impl State8080 {
    pub fn new() -> Self {
        State8080 {
            memory: vec![0; 0x10000],
            ..Default::default()
        }
    }

    pub fn loading_buffer_into_memory_at(self, buffer: Vec<u8>, index: u16) -> Self {
        let range_start = index as usize;
        let range_end = range_start + buffer.len();
        let mut new_memory = self.memory;
        new_memory.splice(range_start..range_end, buffer);

        State8080 {
            memory: new_memory,
            ..self
        }
    }

    pub fn run(self) {
        let mut output = String::new();

        let mut iter = self.memory.iter();
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

        std::fs::write("/Users/prezi/Developer/emu-8080/invaders.txt", output)
            .expect("Failed to write output file");
    }
}
