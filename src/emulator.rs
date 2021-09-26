use std::convert::TryFrom;
use std::num::Wrapping;

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

struct BytePair {
    pub low: u8,
    pub high: u8,
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

impl From<u16> for BytePair {
    fn from(val: u16) -> BytePair {
        let high = (val >> 8) as u8;
        let low = val as u8;

        BytePair { high, low }
    }
}

impl From<BytePair> for u16 {
    fn from(pair: BytePair) -> u16 {
        ((pair.high as u16) << 8) | pair.low as u16
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

    fn reading_next_byte(self) -> (Self, u8) {
        let mut state = self;
        let byte = state.memory[state.pc as usize];
        state.pc += 1;

        (state, byte)
    }

    fn reading_next_pair(self) -> (Self, BytePair) {
        let mut state = self;
        let pair = BytePair {
            low: state.memory[state.pc as usize],
            high: state.memory[state.pc as usize + 1],
        };
        state.pc += 2;

        (state, pair)
    }

    fn setting_logic_flags_a(self) -> Self {
        let mut state = self;

        // Based on data book AC and CY should be cleared
        state.cc.remove(ConditionCodes::CY);
        state.cc.remove(ConditionCodes::AC);

        state.cc.set(ConditionCodes::Z, state.a == 0);
        state.cc.set(ConditionCodes::S, (state.a & 0x80) == 0x80);
        state.cc.set(ConditionCodes::P, parity(state.a));

        state
    }

    fn log_instruction(&self, instruction: Instruction) {
        // pc is incremented after reading it, we should rewind back here for logging
        let instruction_pc = self.pc - 1;
        let mut output_line = format!(
            "{:04x}    {:#04x}    {}",
            instruction_pc,
            instruction.clone() as u8,
            instruction.to_string()
        );

        let mut next_bytes = Vec::new();
        for i in 1..instruction.size() {
            let byte = self.memory[instruction_pc as usize + i as usize];
            next_bytes.push(byte);
        }

        let mut next_bytes_iter = next_bytes.iter();
        if let Some(next) = next_bytes_iter.next() {
            let mut addr_str = format!("{:02x}", next);

            if let Some(next) = next_bytes_iter.next() {
                addr_str = format!("${:02x}{}", next, addr_str);
            } else {
                addr_str = format!("#${}", addr_str);
            }

            output_line = format!("{}    {}", output_line, addr_str);
        }
        println!("{}", output_line);
    }

    fn evaluating_instruction(self, instruction: Instruction) -> Self {
        self.log_instruction(instruction.clone());

        let mut state = self;
        match instruction {
            Instruction::Nop => (),
            Instruction::LxiB => {
                let (new_state, byte_pair) = state.reading_next_pair();

                state = new_state;
                state.b = byte_pair.high;
                state.c = byte_pair.low
            }

            Instruction::StaxB => (),
            Instruction::InxB => (),
            Instruction::InrB => (),
            Instruction::DcrB => (),
            Instruction::MviB => (),
            Instruction::Rlc => (),
            Instruction::DadB => (),
            Instruction::LdaxB => (),
            Instruction::DcxB => (),
            Instruction::InrC => (),
            Instruction::DcrC => (),
            Instruction::MviC => (),
            // 0x0F
            Instruction::Rrc => {
                let x = state.a;
                state.a = ((x & 1) << 7) | (x >> 1);
                state.cc.set(ConditionCodes::CY, (x & 1) == 1);
            }

            Instruction::LxiD => (),
            Instruction::StaxD => (),
            // 0x13
            Instruction::InxD => {
                state.e += 1;
                if state.e == 0 {
                    state.d += 1;
                }
            }
            Instruction::InrD => (),
            Instruction::DcrD => (),
            Instruction::MviD => (),
            Instruction::Ral => (),
            Instruction::DadD => (),
            Instruction::LdaxD => (),
            Instruction::DcxD => (),
            Instruction::InrE => (),
            Instruction::DcrE => (),
            Instruction::MviE => (),

            // 0x1F
            Instruction::Rar => {
                let x = state.a;
                let carry_u8 = state.cc.contains(ConditionCodes::CY) as u8;
                state.a = ((carry_u8 & 1) << 7) | (x >> 1);
                state.cc.set(ConditionCodes::CY, (x & 1) == 1);
            }
            Instruction::LxiH => (),
            Instruction::Shld => (),
            Instruction::InxH => {
                state.l += 1;
                if state.l == 0 {
                    state.h += 1;
                }
            }
            Instruction::InrH => (),
            Instruction::DcrH => (),
            Instruction::MviH => (),
            Instruction::Daa => (),
            Instruction::DadH => (),
            Instruction::Lhld => (),
            Instruction::DcxH => (),
            Instruction::InrL => (),
            Instruction::DcrL => (),
            Instruction::MviL => (),

            // 0x2F
            Instruction::Cma => {
                state.a = !state.a;
            }
            Instruction::LxiSp => (),
            Instruction::Sta => (),
            Instruction::InxSp => (),
            Instruction::InrM => (),
            Instruction::DcrM => (),
            Instruction::MviM => (),
            Instruction::Stc => (),
            Instruction::DadSp => (),
            Instruction::Lda => (),
            Instruction::DcxSp => (),
            Instruction::InrA => (),
            Instruction::DcrA => (),
            Instruction::MviA => (),
            Instruction::Cmc => (),
            Instruction::MovBB => (),
            Instruction::MovBC => (),
            Instruction::MovBD => (),
            Instruction::MovBE => (),
            Instruction::MovBH => (),
            Instruction::MovBL => (),
            Instruction::MovBM => (),
            Instruction::MovBA => (),
            Instruction::MovCB => (),
            Instruction::MovCC => (),
            Instruction::MovCD => (),
            Instruction::MovCE => (),
            Instruction::MovCH => (),
            Instruction::MovCL => (),
            Instruction::MovCM => (),
            Instruction::MovCA => (),
            Instruction::MovDB => (),
            Instruction::MovDC => (),
            Instruction::MovDD => (),
            Instruction::MovDE => (),
            Instruction::MovDH => (),
            Instruction::MovDL => (),
            Instruction::MovDM => (),
            Instruction::MovDA => (),
            Instruction::MovEB => (),
            Instruction::MovEC => (),
            Instruction::MovED => (),
            Instruction::MovEE => (),
            Instruction::MovEH => (),
            Instruction::MovEL => (),
            Instruction::MovEM => (),
            Instruction::MovEA => (),
            Instruction::MovHB => (),
            Instruction::MovHC => (),
            Instruction::MovHD => (),
            Instruction::MovHE => (),
            Instruction::MovHH => (),
            Instruction::MovHL => (),
            Instruction::MovHM => (),
            Instruction::MovHA => (),
            Instruction::MovLB => (),
            Instruction::MovLC => (),
            Instruction::MovLD => (),
            Instruction::MovLE => (),
            Instruction::MovLH => (),
            Instruction::MovLL => (),
            Instruction::MovLM => (),
            Instruction::MovLA => (),
            Instruction::MovMB => (),
            Instruction::MovMC => (),
            Instruction::MovMD => (),
            Instruction::MovME => (),
            Instruction::MovMH => (),
            Instruction::MovML => (),
            Instruction::Hlt => (),
            Instruction::MovMA => (),
            Instruction::MovAB => (),
            Instruction::MovAC => (),
            Instruction::MovAD => (),
            Instruction::MovAE => (),
            Instruction::MovAH => (),
            Instruction::MovAL => (),
            Instruction::MovAM => (),
            Instruction::MovAA => (),
            // 0x80
            Instruction::AddB => {
                let res_precise = state.a as u16 + state.b as u16;
                let res = (res_precise & 0xff) as u8;

                state.a = res;
                state.cc.set(ConditionCodes::Z, res == 0);
                state.cc.set(ConditionCodes::S, res & 0x80 != 0);
                state.cc.set(ConditionCodes::P, parity(res));
                state.cc.set(ConditionCodes::CY, res_precise > 0xff);
            }
            Instruction::AddC => (),
            Instruction::AddD => (),
            Instruction::AddE => (),
            Instruction::AddH => (),
            Instruction::AddL => (),
            // 0x86
            Instruction::AddM => {
                let address: u16 = BytePair {
                    low: state.l,
                    high: state.h,
                }
                .into();

                let res_precise = state.a as u16 + state.memory[address as usize] as u16;
                let res = (res_precise & 0xff) as u8;

                state.a = res;
                state.cc.set(ConditionCodes::Z, res == 0);
                state.cc.set(ConditionCodes::S, res & 0x80 != 0);
                state.cc.set(ConditionCodes::P, parity(res));
                state.cc.set(ConditionCodes::CY, res_precise > 0xff);
            }
            Instruction::AddA => (),
            Instruction::AdcB => (),
            Instruction::AdcC => (),
            Instruction::AdcD => (),
            Instruction::AdcE => (),
            Instruction::AdcH => (),
            Instruction::AdcL => (),
            Instruction::AdcM => (),
            Instruction::AdcA => (),
            Instruction::SubB => (),
            Instruction::SubC => (),
            Instruction::SubD => (),
            Instruction::SubE => (),
            Instruction::SubH => (),
            Instruction::SubL => (),
            Instruction::SubM => (),
            Instruction::SubA => (),
            Instruction::SbbB => (),
            Instruction::SbbC => (),
            Instruction::SbbD => (),
            Instruction::SbbE => (),
            Instruction::SbbH => (),
            Instruction::SbbL => (),
            Instruction::SbbM => (),
            Instruction::SbbA => (),
            Instruction::AnaB => (),
            Instruction::AnaC => (),
            Instruction::AnaD => (),
            Instruction::AnaE => (),
            Instruction::AnaH => (),
            Instruction::AnaL => (),
            Instruction::AnaM => (),
            Instruction::AnaA => (),
            Instruction::XraB => (),
            Instruction::XraC => (),
            Instruction::XraD => (),
            Instruction::XraE => (),
            Instruction::XraH => (),
            Instruction::XraL => (),
            Instruction::XraM => (),
            Instruction::XraA => (),
            Instruction::OraB => (),
            Instruction::OraC => (),
            Instruction::OraD => (),
            Instruction::OraE => (),
            Instruction::OraH => (),
            Instruction::OraL => (),
            Instruction::OraM => (),
            Instruction::OraA => (),
            Instruction::CmpB => (),
            Instruction::CmpC => (),
            Instruction::CmpD => (),
            Instruction::CmpE => (),
            Instruction::CmpH => (),
            Instruction::CmpL => (),
            Instruction::CmpM => (),
            Instruction::CmpA => (),
            Instruction::Rnz => (),
            Instruction::PopB => (),
            // 0xC2
            Instruction::Jnz => {
                let (new_state, pair) = state.reading_next_pair();
                state = new_state;

                if state.cc.contains(ConditionCodes::Z) {
                    state.pc = pair.into();
                }
            }
            // 0xC3
            Instruction::Jmp => {
                let (new_state, pair) = state.reading_next_pair();
                state = new_state;
                state.pc = pair.into();
            }
            Instruction::Cnz => (),
            Instruction::PushB => (),

            // 0xC6
            Instruction::Adi => {
                let (new_state, byte) = state.reading_next_byte();
                state = new_state;

                let res_precise = state.a as u16 + byte as u16;
                let res = (res_precise & 0xff) as u8;

                state.a = res;
                state.cc.set(ConditionCodes::Z, res == 0);
                state.cc.set(ConditionCodes::S, res & 0x80 != 0);
                state.cc.set(ConditionCodes::P, parity(res));
                state.cc.set(ConditionCodes::CY, res_precise > 0xff);
            }
            Instruction::Rst0 => (),
            Instruction::Rz => (),

            // 0xC9
            Instruction::Ret => {
                let low = state.memory[state.sp as usize];
                let high = state.memory[state.sp as usize + 1];
                state.sp += 2;
                state.pc = BytePair { low, high }.into();
            }

            Instruction::Jz => (),
            Instruction::Cz => (),

            // 0xCD
            Instruction::Call => {
                let (new_state, pair) = state.reading_next_pair();
                state = new_state;

                let return_addr = state.pc;
                let return_pair = BytePair::from(return_addr);

                // Use Wrapping since sp starts at 0 and has to wrap on first decrement
                // Planning on implementing a custom wrapper for Wrapping to make dealing with this
                // field easier, but for now not many instructions will need this hopefully.
                let high_mem_addr = (Wrapping(state.sp) - Wrapping(1)).0;
                let low_mem_addr = (Wrapping(state.sp) - Wrapping(2)).0;
                state.memory[high_mem_addr as usize] = return_pair.high;
                state.memory[low_mem_addr as usize] = return_pair.low;
                state.sp = low_mem_addr;

                state.pc = pair.into();
            }
            Instruction::Aci => (),
            Instruction::Rst1 => (),
            Instruction::Rnc => (),
            Instruction::PopD => (),
            Instruction::Jnc => (),
            Instruction::Out => (),
            Instruction::Cnc => (),
            Instruction::PushD => (),
            Instruction::Sui => (),
            Instruction::Rst2 => (),
            Instruction::Rc => (),
            Instruction::Jc => (),
            Instruction::In => (),
            Instruction::Cc => (),
            Instruction::Sbi => (),
            Instruction::Rst3 => (),
            Instruction::Rpo => (),
            Instruction::PopH => (),
            Instruction::Jpo => (),
            Instruction::Xthl => (),
            Instruction::Cpo => (),
            Instruction::PushH => (),
            // 0xE6
            Instruction::Ani => {
                let (new_state, byte) = state.reading_next_byte();
                state = new_state;

                state.a = state.a & byte;
                state = state.setting_logic_flags_a();
            }
            Instruction::Rst4 => (),
            Instruction::Rpe => (),
            Instruction::Pchl => (),
            Instruction::Jpe => (),
            Instruction::Xchg => (),
            Instruction::Cpe => (),
            Instruction::Xri => (),
            Instruction::Rst5 => (),
            Instruction::Rp => (),
            Instruction::PopPsw => (),
            Instruction::Jp => (),
            Instruction::Di => (),
            Instruction::Cp => (),
            Instruction::PushPsw => (),
            Instruction::Ori => (),
            Instruction::Rst6 => (),
            Instruction::Rm => (),
            Instruction::Sphl => (),
            Instruction::Jm => (),
            Instruction::Ei => (),
            Instruction::Cm => (),
            Instruction::Cpi => {
                let (new_state, byte) = state.reading_next_byte();
                state = new_state;

                let res = state.a - byte;

                state.cc.set(ConditionCodes::Z, res == 0);
                state.cc.set(ConditionCodes::S, (res & 0x80) == 0x80);
                state.cc.set(ConditionCodes::P, parity(res));
                state.cc.set(ConditionCodes::CY, state.a < byte);
            }
            Instruction::Rst7 => (),
        }

        // state.pc += instruction.size() as u16 - 1;

        state
    }

    fn evaluating_next(self) -> Self {
        let (mut state, op_code) = self.reading_next_byte();

        match Instruction::try_from(op_code) {
            Ok(instruction) => state = state.evaluating_instruction(instruction),
            Err(_) => println!("Not an instruction: {:#04x}", op_code),
        }

        state
    }

    pub fn run(self) {
        let mut state = self;

        loop {
            state = state.evaluating_next();
        }
    }
}

fn parity(val: u8) -> bool {
    let mut val = val;

    let mut parity = true;

    while val != 0 {
        parity = !parity;
        val = val & (val - 1);
    }

    parity
}
