use std::convert::TryFrom;
use std::fs::read;
use std::path::Path;

use bitflags::bitflags;

use crate::disassembler::Instruction;

bitflags! {
    #[repr(C)]
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

// pub struct InjectedIOHandler<'a>(Box<dyn Fn(u8) + 'a>);

#[repr(C)]
#[derive(Default)]
pub struct State8080 /*<'a>*/ {
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
    pub interrupt_enabled: bool,
    // input_handler: InjectedIOHandler<'a>,
    // output_handler: InjectedIOHandler<'a>,
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

// impl Default for InjectedIOHandler<'_> {
//     fn default() -> Self {
//         InjectedIOHandler(Box::new(|_x| {}))
//     }
// }

impl State8080 /*<'a>*/ {
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

    pub fn loading_file_into_memory_at<P: AsRef<Path>>(self, path: P, index: u16) -> Self {
        let buf = std::fs::read(path).expect("Failed to read file");

        self.loading_buffer_into_memory_at(buf, index)
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

    fn setting_ac_flag_a(self) -> Self {
        let mut state = self;
        state.cc.set(ConditionCodes::AC, state.a > 0x0F);

        state
    }

    fn pushing(self, high: u8, low: u8) -> Self {
        let mut state = self;
        state.memory[state.sp.wrapping_sub(1) as usize] = high;
        state.memory[state.sp.wrapping_sub(2) as usize] = low;
        state.sp = state.sp.wrapping_sub(2);

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

    // pub fn setting_in_handler<H>(self, handler: H) -> Self
    // where
    //     H: Fn(u8) + 'a,
    // {
    //     let mut state = self;
    //     state.input_handler = InjectedIOHandler(Box::new(handler));

    //     state
    // }

    // pub fn setting_out_handler<H>(self, handler: H) -> Self
    // where
    //     H: Fn(u8) + 'a,
    // {
    //     let mut state = self;
    //     state.output_handler = InjectedIOHandler(Box::new(handler));

    //     state
    // }

    pub fn generating_interrupt(self, int_num: u16) -> Self {
        let mut state = self;
        let pc_pair: BytePair = state.pc.into();
        state = state.pushing(pc_pair.high, pc_pair.low);
        state.pc = 8 * int_num;

        state
    }

    fn evaluating_instruction(self, instruction: Instruction) -> Self {
        self.log_instruction(instruction.clone());

        let mut state = self;
        match instruction {
            Instruction::Nop => (),
            // 0x01
            Instruction::LxiB => {
                let (new_state, byte_pair) = state.reading_next_pair();

                state = new_state;
                state.b = byte_pair.high;
                state.c = byte_pair.low
            }
            Instruction::StaxB => (),
            Instruction::InxB => (),
            Instruction::InrB => (),
            // 0x05
            Instruction::DcrB => {
                let res = state.b.wrapping_sub(1);
                state.cc.set(ConditionCodes::Z, res == 0);
                state.cc.set(ConditionCodes::S, (res & 0x80) == 0x80);
                state.cc.set(ConditionCodes::P, parity(res));
                state.cc.set(ConditionCodes::AC, res > 0x0f);
                state.b = res;
            }
            // 0x06
            Instruction::MviB => {
                let (new_state, byte) = state.reading_next_byte();
                state = new_state;
                state.b = byte;
            }
            Instruction::Rlc => (),
            // 0x09
            Instruction::DadB => {
                let hl: u16 = BytePair {
                    high: state.h,
                    low: state.l,
                }
                .into();
                let bc: u16 = BytePair {
                    high: state.b,
                    low: state.c,
                }
                .into();
                let res = (hl as u32).wrapping_add(bc as u32);
                let res_pair = BytePair::from(res as u16);
                state.h = res_pair.high;
                state.l = res_pair.low;
                state.cc.set(ConditionCodes::CY, (res & 0xff00) != 0);
            }
            Instruction::LdaxB => (),
            Instruction::DcxB => (),
            Instruction::InrC => (),
            // 0x0D
            Instruction::DcrC => {
                let res = state.c.wrapping_sub(1);
                state.cc.set(ConditionCodes::Z, res == 0);
                state.cc.set(ConditionCodes::S, (res & 0x80) == 0x80);
                state.cc.set(ConditionCodes::P, parity(res));
                state.cc.set(ConditionCodes::AC, res > 0x0f);
                state.c = res;
            }
            // 0x0E
            Instruction::MviC => {
                let (new_state, byte) = state.reading_next_byte();
                state = new_state;
                state.c = byte;
            }
            // 0x0F
            Instruction::Rrc => {
                let x = state.a;
                state.a = ((x & 1) << 7) | (x >> 1);
                state.cc.set(ConditionCodes::CY, (x & 1) == 1);
            }
            // 0x11
            Instruction::LxiD => {
                let (new_state, byte_pair) = state.reading_next_pair();

                state = new_state;
                state.d = byte_pair.high;
                state.e = byte_pair.low
            }
            Instruction::StaxD => (),
            // 0x13
            Instruction::InxD => {
                state.e = state.e.wrapping_add(1);
                if state.e == 0 {
                    state.d = state.d.wrapping_add(1);
                }
            }
            Instruction::InrD => (),
            Instruction::DcrD => (),
            Instruction::MviD => (),
            Instruction::Ral => (),
            // 0x19
            Instruction::DadD => {
                let hl: u16 = BytePair {
                    high: state.h,
                    low: state.l,
                }
                .into();
                let de: u16 = BytePair {
                    high: state.d,
                    low: state.e,
                }
                .into();
                let res = (hl as u32).wrapping_add(de as u32);
                let res_pair = BytePair::from(res as u16);
                state.h = res_pair.high;
                state.l = res_pair.low;
                state.cc.set(ConditionCodes::CY, (res & 0xff00) != 0);
            }
            //0x1A
            Instruction::LdaxD => {
                let offset: u16 = BytePair {
                    high: state.d,
                    low: state.e,
                }
                .into();
                state.a = state.memory[offset as usize];
            }
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
            // 0x21
            Instruction::LxiH => {
                let (new_state, byte_pair) = state.reading_next_pair();

                state = new_state;
                state.h = byte_pair.high;
                state.l = byte_pair.low
            }
            Instruction::Shld => (),
            // 0x23
            Instruction::InxH => {
                state.l = state.l.wrapping_add(1);
                if state.l == 0 {
                    state.h = state.h.wrapping_add(1);
                }
            }
            Instruction::InrH => (),
            Instruction::DcrH => (),
            // 0x26
            Instruction::MviH => {
                let (new_state, byte) = state.reading_next_byte();
                state = new_state;
                state.h = byte;
            }
            Instruction::Daa => (),
            // 0x29
            Instruction::DadH => {
                let hl: u16 = BytePair {
                    high: state.h,
                    low: state.l,
                }
                .into();
                let res = (hl as u32).wrapping_add(hl as u32);
                let res_pair = BytePair::from(res as u16);
                state.h = res_pair.high;
                state.l = res_pair.low;
                state.cc.set(ConditionCodes::CY, (res & 0xff00) != 0);
            }
            Instruction::Lhld => (),
            Instruction::DcxH => (),
            Instruction::InrL => (),
            Instruction::DcrL => (),
            Instruction::MviL => (),
            // 0x2F
            Instruction::Cma => {
                state.a = !state.a;
            }
            // 0x31
            Instruction::LxiSp => {
                let (new_state, pair) = state.reading_next_pair();
                state = new_state;
                state.sp = pair.into();
            }
            // 0x32
            Instruction::Sta => {
                let (new_state, pair) = state.reading_next_pair();
                let offset: u16 = pair.into();
                state = new_state;
                state.memory[offset as usize] = state.a;
            }
            Instruction::InxSp => (),
            Instruction::InrM => (),
            Instruction::DcrM => (),
            // 0x36
            Instruction::MviM => {
                let (new_state, byte) = state.reading_next_byte();
                state = new_state;

                let offset: u16 = BytePair {
                    high: state.h,
                    low: state.l,
                }
                .into();
                state.memory[offset as usize] = byte;
            }
            Instruction::Stc => (),
            Instruction::DadSp => (),
            // 0x3A
            Instruction::Lda => {
                let (new_state, pair) = state.reading_next_pair();
                let offset: u16 = pair.into();
                state = new_state;
                state.a = state.memory[offset as usize];
            }
            Instruction::DcxSp => (),
            Instruction::InrA => (),
            Instruction::DcrA => (),
            // 0x3E
            Instruction::MviA => {
                let (new_state, byte) = state.reading_next_byte();
                state = new_state;
                state.a = byte;
            }
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
            // 0x56
            Instruction::MovDM => {
                let offset: u16 = BytePair {
                    high: state.h,
                    low: state.l,
                }
                .into();
                state.d = state.memory[offset as usize];
            }
            Instruction::MovDA => (),
            Instruction::MovEB => (),
            Instruction::MovEC => (),
            Instruction::MovED => (),
            Instruction::MovEE => (),
            Instruction::MovEH => (),
            Instruction::MovEL => (),
            // 0x5e
            Instruction::MovEM => {
                let offset: u16 = BytePair {
                    high: state.h,
                    low: state.l,
                }
                .into();
                state.e = state.memory[offset as usize];
            }
            Instruction::MovEA => (),
            Instruction::MovHB => (),
            Instruction::MovHC => (),
            Instruction::MovHD => (),
            Instruction::MovHE => (),
            Instruction::MovHH => (),
            Instruction::MovHL => (),
            // 0x66
            Instruction::MovHM => {
                let offset: u16 = BytePair {
                    high: state.h,
                    low: state.l,
                }
                .into();
                state.h = state.memory[offset as usize];
            }
            Instruction::MovHA => (),
            Instruction::MovLB => (),
            Instruction::MovLC => (),
            Instruction::MovLD => (),
            Instruction::MovLE => (),
            Instruction::MovLH => (),
            Instruction::MovLL => (),
            Instruction::MovLM => (),
            // 0x6F
            Instruction::MovLA => {
                state.l = state.a;
            }
            Instruction::MovMB => (),
            Instruction::MovMC => (),
            Instruction::MovMD => (),
            Instruction::MovME => (),
            Instruction::MovMH => (),
            Instruction::MovML => (),
            Instruction::Hlt => (),
            // 0x77
            Instruction::MovMA => {
                let offset: u16 = BytePair {
                    high: state.h,
                    low: state.l,
                }
                .into();
                state.memory[offset as usize] = state.a;
            }
            Instruction::MovAB => (),
            Instruction::MovAC => (),
            // 0x7A
            Instruction::MovAD => {
                state.a = state.d;
            }
            // 0x7B
            Instruction::MovAE => {
                state.a = state.e;
            }
            // 0x7C
            Instruction::MovAH => {
                state.a = state.h;
            }
            Instruction::MovAL => (),
            // 0x7E
            Instruction::MovAM => {
                let offset: u16 = BytePair {
                    high: state.h,
                    low: state.l,
                }
                .into();
                state.a = state.memory[offset as usize];
            }
            Instruction::MovAA => (),
            // 0x80
            Instruction::AddB => {
                let res_precise = (state.a as u16).wrapping_add(state.b as u16);
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

                let res_precise =
                    (state.a as u16).wrapping_add(state.memory[address as usize] as u16);
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
            // 0xA7
            Instruction::AnaA => {
                let res = state.a & state.a;
                state.a = res;
                state = state.setting_logic_flags_a().setting_ac_flag_a();
            }
            Instruction::XraB => (),
            Instruction::XraC => (),
            Instruction::XraD => (),
            Instruction::XraE => (),
            Instruction::XraH => (),
            Instruction::XraL => (),
            Instruction::XraM => (),
            // 0xAF
            Instruction::XraA => {
                state.a = state.a ^ state.a;
                state = state.setting_logic_flags_a();
            }
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
            // 0xC1
            Instruction::PopB => {
                state.c = state.memory[state.sp as usize];
                state.b = state.memory[state.sp.wrapping_add(1) as usize];
                state.sp = state.sp.wrapping_add(2);
            }
            // 0xC2
            Instruction::Jnz => {
                let (new_state, pair) = state.reading_next_pair();
                state = new_state;

                if !state.cc.contains(ConditionCodes::Z) {
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
            // 0xC5
            Instruction::PushB => {
                let (high, low) = (state.b, state.c);
                state = state.pushing(high, low);
            }
            // 0xC6
            Instruction::Adi => {
                let (new_state, byte) = state.reading_next_byte();
                state = new_state;

                let res_precise = (state.a as u16).wrapping_add(byte as u16);
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
                let high = state.memory[state.sp.wrapping_add(1) as usize];
                state.sp = state.sp.wrapping_add(2);
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

                let high_mem_addr = state.sp.wrapping_sub(1);
                let low_mem_addr = state.sp.wrapping_sub(2);
                state.memory[high_mem_addr as usize] = return_pair.high;
                state.memory[low_mem_addr as usize] = return_pair.low;
                state.sp = low_mem_addr;

                state.pc = pair.into();
            }
            Instruction::Aci => (),
            Instruction::Rst1 => (),
            Instruction::Rnc => (),
            // 0xD1
            Instruction::PopD => {
                state.e = state.memory[state.sp as usize];
                state.d = state.memory[state.sp.wrapping_add(1) as usize];
                state.sp = state.sp.wrapping_add(2);
            }
            Instruction::Jnc => (),
            // 0xD3
            Instruction::Out => {
                let (new_state, b) = state.reading_next_byte();
                state = new_state;
                // (state.output_handler.0)(b);
            }
            Instruction::Cnc => (),
            // 0xD5
            Instruction::PushD => {
                let (high, low) = (state.d, state.e);
                state = state.pushing(high, low);
            }
            Instruction::Sui => (),
            Instruction::Rst2 => (),
            Instruction::Rc => (),
            Instruction::Jc => (),
            // 0xDB
            Instruction::In => {
                let (new_state, b) = state.reading_next_byte();
                state = new_state;
                // (state.input_handler.0)(b);
            }
            Instruction::Cc => (),
            Instruction::Sbi => (),
            Instruction::Rst3 => (),
            Instruction::Rpo => (),
            // 0xE1
            Instruction::PopH => {
                state.l = state.memory[state.sp as usize];
                state.h = state.memory[state.sp.wrapping_add(1) as usize];
                state.sp = state.sp.wrapping_add(2);
            }
            Instruction::Jpo => (),
            Instruction::Xthl => (),
            Instruction::Cpo => (),
            // 0xE5
            Instruction::PushH => {
                let (high, low) = (state.h, state.l);
                state = state.pushing(high, low);
            }
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
            // 0xEB
            Instruction::Xchg => {
                let saved_h = state.h;
                let saved_l = state.l;
                state.h = state.d;
                state.l = state.e;
                state.d = saved_h;
                state.e = saved_l;
            }
            Instruction::Cpe => (),
            Instruction::Xri => (),
            Instruction::Rst5 => (),
            Instruction::Rp => (),
            // 0xF1
            Instruction::PopPsw => {
                state.a = state.memory[state.sp.wrapping_add(1) as usize];
                state.cc.bits = state.memory[state.sp as usize];
                state.sp = state.sp.wrapping_add(2);
            }
            Instruction::Jp => (),
            // 0xF3
            Instruction::Di => {
                state.interrupt_enabled = false;
            }
            Instruction::Cp => (),
            Instruction::PushPsw => {
                let (high, low) = (state.a, state.cc.bits);
                state = state.pushing(high, low);
            }
            Instruction::Ori => (),
            Instruction::Rst6 => (),
            Instruction::Rm => (),
            Instruction::Sphl => (),
            Instruction::Jm => (),
            // 0xFB
            Instruction::Ei => {
                state.interrupt_enabled = true;
            }
            Instruction::Cm => (),
            Instruction::Cpi => {
                let (new_state, byte) = state.reading_next_byte();
                state = new_state;

                let res = state.a.wrapping_sub(byte);

                state.cc.set(ConditionCodes::Z, res == 0);
                state.cc.set(ConditionCodes::S, (res & 0x80) == 0x80);
                state.cc.set(ConditionCodes::P, parity(res));
                state.cc.set(ConditionCodes::CY, state.a < byte);
            }
            Instruction::Rst7 => (),
        }

        state
    }

    pub fn evaluating_next(self) -> Self {
        let (mut state, op_code) = self.reading_next_byte();

        match Instruction::try_from(op_code) {
            Ok(instruction) => state = state.evaluating_instruction(instruction),
            Err(_) => println!("Not an instruction: {:#04x}", op_code),
        }

        state
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
