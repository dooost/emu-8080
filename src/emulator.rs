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

    fn evaluating_op(self) -> Self {
        let mut state = self;

        let op_code = state.memory[state.pc as usize];

        let mut output_line = format!("{:04x}    {:#04x}", state.pc, op_code);

        state.pc += 1;

        match Instruction::try_from(op_code) {
            Ok(instruction) => {
                output_line = format!("{}    {}", output_line, instruction.to_string());

                let mut next_bytes = Vec::new();
                for _i in 1..instruction.size() {
                    let byte = state.memory[state.pc as usize];
                    next_bytes.push(byte);
                    state.pc += 1;
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
                println!("{}", output_line);
            }
            Err(_) => println!("Not an instruction: {:#04x}", op_code),
        }

        state
    }

    pub fn run(self) {
        let mut state = self;

        while true {
            state = state.evaluating_op();
        }
    }
}
