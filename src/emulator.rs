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

        state.pc += instruction.size() as u16 - 1;
        // match instruction {
        //     Instruction::Nop => (),
        //     Instruction::LxiB => {
        //         // state.c = state.memory[state.pc as usize];
        //         // state.c = state.memory[state.pc as usize + 1];
        //         // state.pc += 2;
        //     }
        //     Instruction::StaxB => todo!(),
        //     Instruction::InxB => todo!(),
        //     Instruction::InrB => todo!(),
        //     Instruction::DcrB => todo!(),
        //     Instruction::MviB => todo!(),
        //     Instruction::Rlc => todo!(),
        //     Instruction::DadB => todo!(),
        //     Instruction::LdaxB => todo!(),
        //     Instruction::DcxB => todo!(),
        //     Instruction::InrC => todo!(),
        //     Instruction::DcrC => todo!(),
        //     Instruction::MviC => todo!(),
        //     Instruction::Rrc => todo!(),
        //     Instruction::LxiD => todo!(),
        //     Instruction::StaxD => todo!(),
        //     Instruction::InxD => todo!(),
        //     Instruction::InrD => todo!(),
        //     Instruction::DcrD => todo!(),
        //     Instruction::MviD => todo!(),
        //     Instruction::Ral => todo!(),
        //     Instruction::DadD => todo!(),
        //     Instruction::LdaxD => todo!(),
        //     Instruction::DcxD => todo!(),
        //     Instruction::InrE => todo!(),
        //     Instruction::DcrE => todo!(),
        //     Instruction::MviE => todo!(),
        //     Instruction::Rar => todo!(),
        //     Instruction::LxiH => todo!(),
        //     Instruction::Shld => todo!(),
        //     Instruction::InxH => todo!(),
        //     Instruction::InrH => todo!(),
        //     Instruction::DcrH => todo!(),
        //     Instruction::MviH => todo!(),
        //     Instruction::Daa => todo!(),
        //     Instruction::DadH => todo!(),
        //     Instruction::Lhld => todo!(),
        //     Instruction::DcxH => todo!(),
        //     Instruction::InrL => todo!(),
        //     Instruction::DcrL => todo!(),
        //     Instruction::MviL => todo!(),
        //     Instruction::Cma => todo!(),
        //     Instruction::LxiSp => todo!(),
        //     Instruction::Sta => todo!(),
        //     Instruction::InxSp => todo!(),
        //     Instruction::InrM => todo!(),
        //     Instruction::DcrM => todo!(),
        //     Instruction::MviM => todo!(),
        //     Instruction::Stc => todo!(),
        //     Instruction::DadSp => todo!(),
        //     Instruction::Lda => todo!(),
        //     Instruction::DcxSp => todo!(),
        //     Instruction::InrA => todo!(),
        //     Instruction::DcrA => todo!(),
        //     Instruction::MviA => todo!(),
        //     Instruction::Cmc => todo!(),
        //     Instruction::MovBB => todo!(),
        //     Instruction::MovBC => todo!(),
        //     Instruction::MovBD => todo!(),
        //     Instruction::MovBE => todo!(),
        //     Instruction::MovBH => todo!(),
        //     Instruction::MovBL => todo!(),
        //     Instruction::MovBM => todo!(),
        //     Instruction::MovBA => todo!(),
        //     Instruction::MovCB => todo!(),
        //     Instruction::MovCC => todo!(),
        //     Instruction::MovCD => todo!(),
        //     Instruction::MovCE => todo!(),
        //     Instruction::MovCH => todo!(),
        //     Instruction::MovCL => todo!(),
        //     Instruction::MovCM => todo!(),
        //     Instruction::MovCA => todo!(),
        //     Instruction::MovDB => todo!(),
        //     Instruction::MovDC => todo!(),
        //     Instruction::MovDD => todo!(),
        //     Instruction::MovDE => todo!(),
        //     Instruction::MovDH => todo!(),
        //     Instruction::MovDL => todo!(),
        //     Instruction::MovDM => todo!(),
        //     Instruction::MovDA => todo!(),
        //     Instruction::MovEB => todo!(),
        //     Instruction::MovEC => todo!(),
        //     Instruction::MovED => todo!(),
        //     Instruction::MovEE => todo!(),
        //     Instruction::MovEH => todo!(),
        //     Instruction::MovEL => todo!(),
        //     Instruction::MovEM => todo!(),
        //     Instruction::MovEA => todo!(),
        //     Instruction::MovHB => todo!(),
        //     Instruction::MovHC => todo!(),
        //     Instruction::MovHD => todo!(),
        //     Instruction::MovHE => todo!(),
        //     Instruction::MovHH => todo!(),
        //     Instruction::MovHL => todo!(),
        //     Instruction::MovHM => todo!(),
        //     Instruction::MovHA => todo!(),
        //     Instruction::MovLB => todo!(),
        //     Instruction::MovLC => todo!(),
        //     Instruction::MovLD => todo!(),
        //     Instruction::MovLE => todo!(),
        //     Instruction::MovLH => todo!(),
        //     Instruction::MovLL => todo!(),
        //     Instruction::MovLM => todo!(),
        //     Instruction::MovLA => todo!(),
        //     Instruction::MovMB => todo!(),
        //     Instruction::MovMC => todo!(),
        //     Instruction::MovMD => todo!(),
        //     Instruction::MovME => todo!(),
        //     Instruction::MovMH => todo!(),
        //     Instruction::MovML => todo!(),
        //     Instruction::Hlt => todo!(),
        //     Instruction::MovMA => todo!(),
        //     Instruction::MovAB => todo!(),
        //     Instruction::MovAC => todo!(),
        //     Instruction::MovAD => todo!(),
        //     Instruction::MovAE => todo!(),
        //     Instruction::MovAH => todo!(),
        //     Instruction::MovAL => todo!(),
        //     Instruction::MovAM => todo!(),
        //     Instruction::MovAA => todo!(),
        //     Instruction::AddB => todo!(),
        //     Instruction::AddC => todo!(),
        //     Instruction::AddD => todo!(),
        //     Instruction::AddE => todo!(),
        //     Instruction::AddH => todo!(),
        //     Instruction::AddL => todo!(),
        //     Instruction::AddM => todo!(),
        //     Instruction::AddA => todo!(),
        //     Instruction::AdcB => todo!(),
        //     Instruction::AdcC => todo!(),
        //     Instruction::AdcD => todo!(),
        //     Instruction::AdcE => todo!(),
        //     Instruction::AdcH => todo!(),
        //     Instruction::AdcL => todo!(),
        //     Instruction::AdcM => todo!(),
        //     Instruction::AdcA => todo!(),
        //     Instruction::SubB => todo!(),
        //     Instruction::SubC => todo!(),
        //     Instruction::SubD => todo!(),
        //     Instruction::SubE => todo!(),
        //     Instruction::SubH => todo!(),
        //     Instruction::SubL => todo!(),
        //     Instruction::SubM => todo!(),
        //     Instruction::SubA => todo!(),
        //     Instruction::SbbB => todo!(),
        //     Instruction::SbbC => todo!(),
        //     Instruction::SbbD => todo!(),
        //     Instruction::SbbE => todo!(),
        //     Instruction::SbbH => todo!(),
        //     Instruction::SbbL => todo!(),
        //     Instruction::SbbM => todo!(),
        //     Instruction::SbbA => todo!(),
        //     Instruction::AnaB => todo!(),
        //     Instruction::AnaC => todo!(),
        //     Instruction::AnaD => todo!(),
        //     Instruction::AnaE => todo!(),
        //     Instruction::AnaH => todo!(),
        //     Instruction::AnaL => todo!(),
        //     Instruction::AnaM => todo!(),
        //     Instruction::AnaA => todo!(),
        //     Instruction::XraB => todo!(),
        //     Instruction::XraC => todo!(),
        //     Instruction::XraD => todo!(),
        //     Instruction::XraE => todo!(),
        //     Instruction::XraH => todo!(),
        //     Instruction::XraL => todo!(),
        //     Instruction::XraM => todo!(),
        //     Instruction::XraA => todo!(),
        //     Instruction::OraB => todo!(),
        //     Instruction::OraC => todo!(),
        //     Instruction::OraD => todo!(),
        //     Instruction::OraE => todo!(),
        //     Instruction::OraH => todo!(),
        //     Instruction::OraL => todo!(),
        //     Instruction::OraM => todo!(),
        //     Instruction::OraA => todo!(),
        //     Instruction::CmpB => todo!(),
        //     Instruction::CmpC => todo!(),
        //     Instruction::CmpD => todo!(),
        //     Instruction::CmpE => todo!(),
        //     Instruction::CmpH => todo!(),
        //     Instruction::CmpL => todo!(),
        //     Instruction::CmpM => todo!(),
        //     Instruction::CmpA => todo!(),
        //     Instruction::Rnz => todo!(),
        //     Instruction::PopB => todo!(),
        //     Instruction::Jnz => todo!(),
        //     Instruction::Jmp => todo!(),
        //     Instruction::Cnz => todo!(),
        //     Instruction::PushB => todo!(),
        //     Instruction::Adi => todo!(),
        //     Instruction::Rst0 => todo!(),
        //     Instruction::Rz => todo!(),
        //     Instruction::Ret => todo!(),
        //     Instruction::Jz => todo!(),
        //     Instruction::Cz => todo!(),
        //     Instruction::Call => todo!(),
        //     Instruction::Aci => todo!(),
        //     Instruction::Rst1 => todo!(),
        //     Instruction::Rnc => todo!(),
        //     Instruction::PopD => todo!(),
        //     Instruction::Jnc => todo!(),
        //     Instruction::Out => todo!(),
        //     Instruction::Cnc => todo!(),
        //     Instruction::PushD => todo!(),
        //     Instruction::Sui => todo!(),
        //     Instruction::Rst2 => todo!(),
        //     Instruction::Rc => todo!(),
        //     Instruction::Jc => todo!(),
        //     Instruction::In => todo!(),
        //     Instruction::Cc => todo!(),
        //     Instruction::Sbi => todo!(),
        //     Instruction::Rst3 => todo!(),
        //     Instruction::Rpo => todo!(),
        //     Instruction::PopH => todo!(),
        //     Instruction::Jpo => todo!(),
        //     Instruction::Xthl => todo!(),
        //     Instruction::Cpo => todo!(),
        //     Instruction::PushH => todo!(),
        //     Instruction::Ani => todo!(),
        //     Instruction::Rst4 => todo!(),
        //     Instruction::Rpe => todo!(),
        //     Instruction::Pchl => todo!(),
        //     Instruction::Jpe => todo!(),
        //     Instruction::Xchg => todo!(),
        //     Instruction::Cpe => todo!(),
        //     Instruction::Xri => todo!(),
        //     Instruction::Rst5 => todo!(),
        //     Instruction::Rp => todo!(),
        //     Instruction::PopPsw => todo!(),
        //     Instruction::Jp => todo!(),
        //     Instruction::Di => todo!(),
        //     Instruction::Cp => todo!(),
        //     Instruction::PushPsw => todo!(),
        //     Instruction::Ori => todo!(),
        //     Instruction::Rst6 => todo!(),
        //     Instruction::Rm => todo!(),
        //     Instruction::Sphl => todo!(),
        //     Instruction::Jm => todo!(),
        //     Instruction::Ei => todo!(),
        //     Instruction::Cm => todo!(),
        //     Instruction::Cpi => todo!(),
        //     Instruction::Rst7 => todo!(),
        // }

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

        while true {
            state = state.evaluating_next();
        }
    }
}
