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

    fn evaluating_instruction(self, instruction: Instruction) -> Self {
        self.log_instruction(instruction.clone());

        let mut state = self;
        match instruction {
            Instruction::Nop => (),
            Instruction::LxiB => {
                state.c = state.memory[state.pc as usize];
                state.c = state.memory[state.pc as usize + 1];
                state.pc += 2;
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
            Instruction::Rrc => (),
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
            Instruction::Rar => (),
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
            Instruction::Cma => (),
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
                let index = ((state.h as u16) << 8) | state.l as u16;
                let res_precise = state.a as u16 + state.memory[index as usize] as u16;
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
            Instruction::Jnz => (),
            Instruction::Jmp => (),
            Instruction::Cnz => (),
            Instruction::PushB => (),

            // 0xC6
            Instruction::Adi => {
                let res_precise = state.a as u16 + state.memory[state.pc as usize] as u16;
                let res = (res_precise & 0xff) as u8;

                state.pc += 1;

                state.a = res;
                state.cc.set(ConditionCodes::Z, res == 0);
                state.cc.set(ConditionCodes::S, res & 0x80 != 0);
                state.cc.set(ConditionCodes::P, parity(res));
                state.cc.set(ConditionCodes::CY, res_precise > 0xff);
            }
            Instruction::Rst0 => (),
            Instruction::Rz => (),
            Instruction::Ret => (),
            Instruction::Jz => (),
            Instruction::Cz => (),
            Instruction::Call => (),
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
            Instruction::Ani => (),
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
            Instruction::Cpi => (),
            Instruction::Rst7 => (),
        }

        // state.pc += instruction.size() as u16 - 1;

        state
    }

    fn evaluating_next(self) -> Self {
        let mut state = self;
        let op_code = state.memory[state.pc as usize];
        state.pc += 1;

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
