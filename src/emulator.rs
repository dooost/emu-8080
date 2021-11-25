use std::convert::TryFrom;
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
    pub cc: ConditionCodes,
    pub interrupt_enabled: bool,
    memory: Vec<u8>,
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

    pub fn setting_memory_at(self, byte: u8, index: u16) -> Self {
        let mut new_memory = self.memory;
        new_memory[index as usize] = byte;

        State8080 {
            memory: new_memory,
            ..self
        }
    }

    fn bc(&self) -> BytePair {
        BytePair {
            high: self.b,
            low: self.c,
        }
    }

    fn de(&self) -> BytePair {
        BytePair {
            high: self.d,
            low: self.e,
        }
    }

    fn hl(&self) -> BytePair {
        BytePair {
            high: self.h,
            low: self.l,
        }
    }

    fn reading_next_byte(self) -> (Self, u8) {
        let mut state = self;
        let byte = state.memory[state.pc as usize];
        state.pc = state.pc.wrapping_add(1);

        (state, byte)
    }

    fn reading_next_pair(self) -> (Self, BytePair) {
        let mut state = self;
        let pair = BytePair {
            low: state.memory[state.pc as usize],
            high: state.memory[state.pc as usize + 1],
        };
        state.pc = state.pc.wrapping_add(2);

        (state, pair)
    }

    fn setting_flag(self, flag: ConditionCodes, value: bool) -> Self {
        let mut state = self;
        state.cc.set(flag, value);
        state
    }

    fn setting_logic_flags_a(self) -> Self {
        let a = self.a;

        self.setting_z_flag(a)
            .setting_s_flag(a)
            .setting_p_flag(a)
            .setting_flag(ConditionCodes::CY, false)
            .setting_flag(ConditionCodes::AC, false)
    }

    fn setting_ac_flag_a(self) -> Self {
        let a = self.a;
        self.setting_ac_flag(a)
    }

    fn setting_z_flag(self, value: u8) -> Self {
        self.setting_flag(ConditionCodes::Z, value == 0)
    }

    fn setting_s_flag(self, value: u8) -> Self {
        self.setting_flag(ConditionCodes::S, (value & 0x80) == 0x80)
    }

    fn setting_p_flag(self, value: u8) -> Self {
        self.setting_flag(ConditionCodes::P, parity(value))
    }

    fn setting_ac_flag(self, value: u8) -> Self {
        self.setting_flag(ConditionCodes::AC, value > 0x0f)
    }

    fn setting_zspac_flags(self, value: u8) -> Self {
        self.setting_z_flag(value)
            .setting_s_flag(value)
            .setting_p_flag(value)
            .setting_ac_flag(value)
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
        state.interrupt_enabled = false;

        state
    }

    fn setting_bc(self, pair: BytePair) -> Self {
        Self {
            b: pair.high,
            c: pair.low,
            ..self
        }
    }

    fn setting_de(self, pair: BytePair) -> Self {
        Self {
            d: pair.high,
            e: pair.low,
            ..self
        }
    }

    fn setting_hl(self, pair: BytePair) -> Self {
        Self {
            h: pair.high,
            l: pair.low,
            ..self
        }
    }

    fn setting_a(self, a: u8) -> Self {
        Self { a, ..self }
    }

    fn setting_b(self, b: u8) -> Self {
        Self { b, ..self }
    }

    fn setting_c(self, c: u8) -> Self {
        Self { c, ..self }
    }

    fn setting_d(self, d: u8) -> Self {
        Self { d, ..self }
    }

    fn setting_e(self, e: u8) -> Self {
        Self { e, ..self }
    }

    fn setting_h(self, h: u8) -> Self {
        Self { h, ..self }
    }

    fn setting_l(self, l: u8) -> Self {
        Self { l, ..self }
    }

    fn setting_sp(self, sp: u16) -> Self {
        Self { sp, ..self }
    }

    fn evaluating_instruction(self, instruction: Instruction) -> Self {
        #[cfg(feature = "logging")]
        #[cfg(not(feature = "diagsupport"))]
        self.log_instruction(instruction.clone());

        // let state;
        match instruction {
            // 0x00
            Instruction::Nop
            | Instruction::Nop1 // 0x08
            | Instruction::Nop2 // 0x10
            | Instruction::Nop3 // 0x18
            | Instruction::Nop4 // 0x20
            | Instruction::Nop5 // 0x28
            | Instruction::Nop6 // 0x30
            | Instruction::Nop7 => self, // 0x38

            // 0x01
            Instruction::LxiB => {
                let (new_state, byte_pair) = self.reading_next_pair();

                new_state.setting_bc(byte_pair)
            }
            // 0x11
            Instruction::LxiD => {
                let (new_state, byte_pair) = self.reading_next_pair();

                new_state.setting_de(byte_pair)
            }
            // 0x21
            Instruction::LxiH => {
                let (new_state, byte_pair) = self.reading_next_pair();

                new_state.setting_hl(byte_pair)
            }
            // 0x31
            Instruction::LxiSp => {
                let (new_state, pair) = self.reading_next_pair();

                new_state.setting_sp(pair.into())
            }

            // 0x05
            Instruction::DcrB => {
                let res = self.b.wrapping_sub(1);

                self.setting_b(res).setting_zspac_flags(res)
            }
            // 0x0D
            Instruction::DcrC => {
                let res = self.c.wrapping_sub(1);

                self.setting_c(res).setting_zspac_flags(res)
            }
            // 0x15
            Instruction::DcrD => {
                let res = self.d.wrapping_sub(1);

                self.setting_d(res).setting_zspac_flags(res)
            }
            // 0x1D
            Instruction::DcrE => {
                let res = self.e.wrapping_sub(1);

                self.setting_e(res).setting_zspac_flags(res)
            }
            // 0x25
            Instruction::DcrH => {
                let res = self.h.wrapping_sub(1);

                self.setting_h(res).setting_zspac_flags(res)
            }
            // 0x2D
            Instruction::DcrL => {
                let res = self.l.wrapping_sub(1);

                self.setting_l(res).setting_zspac_flags(res)
            }
            // 0x35
            Instruction::DcrM => {
                let offset: u16 = self.hl().into();
                let res = self.memory[offset as usize].wrapping_sub(1);

                self.setting_memory_at(res, offset).setting_zspac_flags(res)
            }
            // 0x3D
            Instruction::DcrA => {
                let res = self.a.wrapping_sub(1);

                self.setting_a(res).setting_zspac_flags(res)
            }

            Instruction::StaxB => self,
            Instruction::InxB => self,
            Instruction::InrB => self,

            // 0x3E
            Instruction::MviA => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.setting_a(byte)
            }
            // 0x06
            Instruction::MviB => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.setting_b(byte)
            }
            // 0x0E
            Instruction::MviC => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.setting_c(byte)
            }
            // 0x16
            Instruction::MviD => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.setting_d(byte)
            }
            // 0x1E
            Instruction::MviE => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.setting_e(byte)
            }
            // 0x26
            Instruction::MviH => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.setting_h(byte)
            }
            // 0x2E
            Instruction::MviL => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.setting_l(byte)
            }
            // 0x36
            Instruction::MviM => {
                let (new_state, byte) = self.reading_next_byte();
                let offset: u16 = BytePair {
                    high: new_state.h,
                    low: new_state.l,
                }
                .into();

                new_state.setting_memory_at(byte, offset)
            }

            // 0x09
            Instruction::DadB => {
                let hl: u16 = self.hl().into();
                let bc: u16 = self.bc().into();
                let res = (hl as u32).wrapping_add(bc as u32);
                let res_pair = BytePair::from(res as u16);
                let new_state = self.setting_hl(res_pair);

                new_state.setting_flag(ConditionCodes::CY, (res & 0xff00) != 0)
            }
            // 0x19
            Instruction::DadD => {
                let hl: u16 = self.hl().into();
                let de: u16 = self.de().into();
                let res = (hl as u32).wrapping_add(de as u32);
                let res_pair = BytePair::from(res as u16);
                let new_state = self.setting_hl(res_pair);

                new_state.setting_flag(ConditionCodes::CY, (res & 0xff00) != 0)
            }
            // 0x29
            Instruction::DadH => {
                let hl: u16 = self.hl().into();
                let res = (hl as u32).wrapping_add(hl as u32);
                let res_pair = BytePair::from(res as u16);
                let new_state = self.setting_hl(res_pair);

                new_state.setting_flag(ConditionCodes::CY, (res & 0xff00) != 0)
            }
            // 0x39
            Instruction::DadSp => {
                let hl: u16 = self.hl().into();
                let sp = self.sp;
                let res = (hl as u32).wrapping_add(sp as u32);
                let res_pair = BytePair::from(res as u16);
                let new_state = self.setting_hl(res_pair);

                new_state.setting_flag(ConditionCodes::CY, (res & 0xff00) != 0)
            }

            Instruction::Rlc => self,

            Instruction::LdaxB => self,
            Instruction::DcxB => self,
            Instruction::InrC => self,

            // 0x0F
            Instruction::Rrc => {
                let x = self.a;
                let mut cc = self.cc.clone();
                let a = ((x & 1) << 7) | (x >> 1);
                cc.set(ConditionCodes::CY, (x & 1) == 1);
                Self { a, cc, ..self }
            }

            Instruction::StaxD => self,
            // 0x13
            Instruction::InxD => {
                let e = self.e.wrapping_add(1);
                let mut d: Option<u8> = None;
                if e == 0 {
                    d = Some(self.d.wrapping_add(1));
                }
                Self {
                    e,
                    d: d.unwrap_or(self.d),
                    ..self
                }
            }
            Instruction::InrD => self,
            Instruction::Ral => self,

            //0x1A
            Instruction::LdaxD => {
                let offset: u16 = BytePair {
                    high: self.d,
                    low: self.e,
                }
                .into();

                Self {
                    a: self.memory[offset as usize],
                    ..self
                }
            }
            Instruction::DcxD => self,
            Instruction::InrE => self,
            // 0x1F
            Instruction::Rar => {
                let x = self.a;
                let carry_u8 = self.cc.contains(ConditionCodes::CY) as u8;
                let mut cc = self.cc.clone();
                let a = ((carry_u8 & 1) << 7) | (x >> 1);
                cc.set(ConditionCodes::CY, (x & 1) == 1);
                Self { a, cc, ..self }
            }

            Instruction::Shld => self,
            // 0x23
            Instruction::InxH => {
                let l = self.l.wrapping_add(1);
                let mut h: Option<u8> = None;
                if l == 0 {
                    h = Some(self.h.wrapping_add(1));
                }
                Self {
                    l,
                    h: h.unwrap_or(self.h),
                    ..self
                }
            }
            Instruction::InrH => self,
            Instruction::Daa => self,

            Instruction::Lhld => self,
            Instruction::DcxH => self,
            Instruction::InrL => self,
            // 0x2F
            Instruction::Cma => Self { a: !self.a, ..self },

            // 0x32
            Instruction::Sta => {
                let (new_state, pair) = self.reading_next_pair();
                let offset: u16 = pair.into();
                let byte = new_state.a;
                new_state.setting_memory_at(byte, offset)
            }
            Instruction::InxSp => self,
            Instruction::InrM => self,

            Instruction::Stc => self,
            // 0x3A
            Instruction::Lda => {
                let (new_state, pair) = self.reading_next_pair();
                let offset: u16 = pair.into();
                Self {
                    a: new_state.memory[offset as usize],
                    ..new_state
                }
            }
            Instruction::DcxSp => self,
            Instruction::InrA => self,

            Instruction::Cmc => self,
            Instruction::MovBB => self,
            Instruction::MovBC => self,
            Instruction::MovBD => self,
            Instruction::MovBE => self,
            Instruction::MovBH => self,
            Instruction::MovBL => self,
            Instruction::MovBM => self,
            Instruction::MovBA => self,
            Instruction::MovCB => self,
            Instruction::MovCC => self,
            Instruction::MovCD => self,
            Instruction::MovCE => self,
            Instruction::MovCH => self,
            Instruction::MovCL => self,
            Instruction::MovCM => self,
            Instruction::MovCA => self,
            Instruction::MovDB => self,
            Instruction::MovDC => self,
            Instruction::MovDD => self,
            Instruction::MovDE => self,
            Instruction::MovDH => self,
            Instruction::MovDL => self,
            // 0x56
            Instruction::MovDM => {
                let offset: u16 = BytePair {
                    high: self.h,
                    low: self.l,
                }
                .into();
                Self {
                    d: self.memory[offset as usize],
                    ..self
                }
            }
            Instruction::MovDA => self,
            Instruction::MovEB => self,
            Instruction::MovEC => self,
            Instruction::MovED => self,
            Instruction::MovEE => self,
            Instruction::MovEH => self,
            Instruction::MovEL => self,
            // 0x5e
            Instruction::MovEM => {
                let offset: u16 = BytePair {
                    high: self.h,
                    low: self.l,
                }
                .into();
                Self {
                    e: self.memory[offset as usize],
                    ..self
                }
            }
            Instruction::MovEA => self,
            Instruction::MovHB => self,
            Instruction::MovHC => self,
            Instruction::MovHD => self,
            Instruction::MovHE => self,
            Instruction::MovHH => self,
            Instruction::MovHL => self,
            // 0x66
            Instruction::MovHM => {
                let offset: u16 = BytePair {
                    high: self.h,
                    low: self.l,
                }
                .into();
                Self {
                    h: self.memory[offset as usize],
                    ..self
                }
            }
            Instruction::MovHA => self,
            Instruction::MovLB => self,
            Instruction::MovLC => self,
            Instruction::MovLD => self,
            Instruction::MovLE => self,
            Instruction::MovLH => self,
            Instruction::MovLL => self,
            Instruction::MovLM => self,
            // 0x6F
            Instruction::MovLA => Self { l: self.a, ..self },
            Instruction::MovMB => self,
            Instruction::MovMC => self,
            Instruction::MovMD => self,
            Instruction::MovME => self,
            Instruction::MovMH => self,
            Instruction::MovML => self,
            Instruction::Hlt => self,
            // 0x77
            Instruction::MovMA => {
                let offset: u16 = BytePair {
                    high: self.h,
                    low: self.l,
                }
                .into();
                let byte = self.a;
                self.setting_memory_at(byte, offset)
            }
            Instruction::MovAB => self,
            Instruction::MovAC => self,
            // 0x7A
            Instruction::MovAD => Self { a: self.d, ..self },
            // 0x7B
            Instruction::MovAE => Self { a: self.e, ..self },
            // 0x7C
            Instruction::MovAH => Self { a: self.h, ..self },
            Instruction::MovAL => self,
            // 0x7E
            Instruction::MovAM => {
                let offset: u16 = BytePair {
                    high: self.h,
                    low: self.l,
                }
                .into();
                Self {
                    a: self.memory[offset as usize],
                    ..self
                }
            }
            Instruction::MovAA => self,
            // 0x80
            Instruction::AddB => {
                let mut cc = self.cc.clone();
                let res_precise = (self.a as u16).wrapping_add(self.b as u16);
                let res = (res_precise & 0xff) as u8;
                cc.set(ConditionCodes::Z, res == 0);
                cc.set(ConditionCodes::S, res & 0x80 != 0);
                cc.set(ConditionCodes::P, parity(res));
                cc.set(ConditionCodes::CY, res_precise > 0xff);

                Self { a: res, cc, ..self }
            }
            Instruction::AddC => self,
            Instruction::AddD => self,
            Instruction::AddE => self,
            Instruction::AddH => self,
            Instruction::AddL => self,
            // 0x86
            Instruction::AddM => {
                let mut cc = self.cc.clone();
                let address: u16 = BytePair {
                    low: self.l,
                    high: self.h,
                }
                .into();

                let res_precise =
                    (self.a as u16).wrapping_add(self.memory[address as usize] as u16);
                let res = (res_precise & 0xff) as u8;
                cc.set(ConditionCodes::Z, res == 0);
                cc.set(ConditionCodes::S, res & 0x80 != 0);
                cc.set(ConditionCodes::P, parity(res));
                cc.set(ConditionCodes::CY, res_precise > 0xff);

                Self { a: res, cc, ..self }
            }
            Instruction::AddA => self,
            Instruction::AdcB => self,
            Instruction::AdcC => self,
            Instruction::AdcD => self,
            Instruction::AdcE => self,
            Instruction::AdcH => self,
            Instruction::AdcL => self,
            Instruction::AdcM => self,
            Instruction::AdcA => self,
            Instruction::SubB => self,
            Instruction::SubC => self,
            Instruction::SubD => self,
            Instruction::SubE => self,
            Instruction::SubH => self,
            Instruction::SubL => self,
            Instruction::SubM => self,
            Instruction::SubA => self,
            Instruction::SbbB => self,
            Instruction::SbbC => self,
            Instruction::SbbD => self,
            Instruction::SbbE => self,
            Instruction::SbbH => self,
            Instruction::SbbL => self,
            Instruction::SbbM => self,
            Instruction::SbbA => self,
            Instruction::AnaB => self,
            Instruction::AnaC => self,
            Instruction::AnaD => self,
            Instruction::AnaE => self,
            Instruction::AnaH => self,
            Instruction::AnaL => self,
            Instruction::AnaM => self,
            // 0xA7
            Instruction::AnaA => {
                let new_state = Self {
                    a: self.a & self.a,
                    ..self
                };

                new_state.setting_logic_flags_a().setting_ac_flag_a()
            }
            Instruction::XraB => self,
            Instruction::XraC => self,
            Instruction::XraD => self,
            Instruction::XraE => self,
            Instruction::XraH => self,
            Instruction::XraL => self,
            Instruction::XraM => self,
            // 0xAF
            Instruction::XraA => {
                let new_state = Self {
                    a: self.a ^ self.a,
                    ..self
                };

                new_state.setting_logic_flags_a()
            }
            Instruction::OraB => self,
            Instruction::OraC => self,
            Instruction::OraD => self,
            Instruction::OraE => self,
            Instruction::OraH => self,
            Instruction::OraL => self,
            Instruction::OraM => self,
            Instruction::OraA => self,
            Instruction::CmpB => self,
            Instruction::CmpC => self,
            Instruction::CmpD => self,
            Instruction::CmpE => self,
            Instruction::CmpH => self,
            Instruction::CmpL => self,
            Instruction::CmpM => self,
            Instruction::CmpA => self,
            Instruction::Rnz => self,
            // 0xC1
            Instruction::PopB => Self {
                c: self.memory[self.sp as usize],
                b: self.memory[self.sp.wrapping_add(1) as usize],
                sp: self.sp.wrapping_add(2),
                ..self
            },
            // 0xC2
            Instruction::Jnz => {
                let (new_state, pair) = self.reading_next_pair();

                if !new_state.cc.contains(ConditionCodes::Z) {
                    Self {
                        pc: pair.into(),
                        ..new_state
                    }
                } else {
                    new_state
                }
            }
            // 0xC3
            Instruction::Jmp => {
                let (new_state, pair) = self.reading_next_pair();
                Self {
                    pc: pair.into(),
                    ..new_state
                }
            }
            Instruction::Cnz => self,
            // 0xC5
            Instruction::PushB => {
                let (high, low) = (self.b, self.c);
                self.pushing(high, low)
            }
            // 0xC6
            Instruction::Adi => {
                let mut cc = self.cc.clone();
                let (new_state, byte) = self.reading_next_byte();

                let res_precise = (new_state.a as u16).wrapping_add(byte as u16);
                let res = (res_precise & 0xff) as u8;
                cc.set(ConditionCodes::Z, res == 0);
                cc.set(ConditionCodes::S, res & 0x80 != 0);
                cc.set(ConditionCodes::P, parity(res));
                cc.set(ConditionCodes::CY, res_precise > 0xff);

                Self {
                    a: res,
                    cc,
                    ..new_state
                }
            }
            Instruction::Rst0 => self,
            Instruction::Rz => self,
            // 0xC9
            Instruction::Ret => {
                let low = self.memory[self.sp as usize];
                let high = self.memory[self.sp.wrapping_add(1) as usize];

                Self {
                    sp: self.sp.wrapping_add(2),
                    pc: BytePair { low, high }.into(),
                    ..self
                }
            }
            Instruction::Jz => self,
            Instruction::Cz => self,
            // 0xCD
            Instruction::Call => {
                let (new_state, pair) = self.reading_next_pair();

                let addr: u16 = pair.into();
                if cfg!(feature = "diagsupport") && addr == 5 {
                    if new_state.c == 9 {
                        let offset: u16 = BytePair {
                            high: new_state.d,
                            low: new_state.e,
                        }
                        .into();
                        new_state.memory[(offset as usize + 3)..]
                            .iter()
                            .take_while(|c| **c != b'$')
                            .map(|c| *c)
                            .for_each(|c| print!("{}", c as char));
                        println!();
                    } else if new_state.c == 2 {
                        println!("print char routine called");
                    }

                    new_state
                } else if cfg!(feature = "diagsupport") && addr == 0 {
                    panic!("Diag hit call 0");
                } else {
                    let return_addr = new_state.pc;
                    let return_pair = BytePair::from(return_addr);

                    let high_mem_addr = new_state.sp.wrapping_sub(1);
                    let low_mem_addr = new_state.sp.wrapping_sub(2);

                    let new_state = Self {
                        pc: addr,
                        ..new_state
                    };

                    new_state
                        .setting_memory_at(return_pair.high, high_mem_addr)
                        .setting_memory_at(return_pair.low, low_mem_addr)
                }
            }
            Instruction::Aci => self,
            Instruction::Rst1 => self,
            Instruction::Rnc => self,
            // 0xD1
            Instruction::PopD => Self {
                e: self.memory[self.sp as usize],
                d: self.memory[self.sp.wrapping_add(1) as usize],
                sp: self.sp.wrapping_add(2),
                ..self
            },
            Instruction::Jnc => self,
            // 0xD3
            Instruction::Out => {
                let (new_state, b) = self.reading_next_byte();
                new_state
                // (state.output_handler.0)(b);
            }
            Instruction::Cnc => self,
            // 0xD5
            Instruction::PushD => {
                let (high, low) = (self.d, self.e);
                self.pushing(high, low)
            }
            Instruction::Sui => self,
            Instruction::Rst2 => self,
            Instruction::Rc => self,
            Instruction::Jc => self,
            // 0xDB
            Instruction::In => {
                let (new_state, b) = self.reading_next_byte();
                new_state
                // (state.input_handler.0)(b);
            }
            Instruction::Cc => self,
            Instruction::Sbi => self,
            Instruction::Rst3 => self,
            Instruction::Rpo => self,
            // 0xE1
            Instruction::PopH => Self {
                l: self.memory[self.sp as usize],
                h: self.memory[self.sp.wrapping_add(1) as usize],
                sp: self.sp.wrapping_add(2),
                ..self
            },
            Instruction::Jpo => self,
            Instruction::Xthl => self,
            Instruction::Cpo => self,
            // 0xE5
            Instruction::PushH => {
                let (high, low) = (self.h, self.l);
                self.pushing(high, low)
            }
            // 0xE6
            Instruction::Ani => {
                let (new_state, byte) = self.reading_next_byte();
                let new_state = Self {
                    a: new_state.a & byte,
                    ..new_state
                };

                new_state.setting_logic_flags_a()
            }
            Instruction::Rst4 => self,
            Instruction::Rpe => self,
            Instruction::Pchl => self,
            Instruction::Jpe => self,

            // 0xEB
            Instruction::Xchg => Self {
                h: self.d,
                l: self.e,
                d: self.h,
                e: self.l,
                ..self
            },

            Instruction::Cpe => self,
            Instruction::Xri => self,
            Instruction::Rst5 => self,
            Instruction::Rp => self,
            // 0xF1
            Instruction::PopPsw => Self {
                a: self.memory[self.sp.wrapping_add(1) as usize],
                sp: self.sp.wrapping_add(2),
                cc: ConditionCodes::from_bits_truncate(self.memory[self.sp as usize]),
                ..self
            },
            Instruction::Jp => self,

            Instruction::Cp => self,

            Instruction::PushPsw => {
                let (high, low) = (self.a, self.cc.bits);

                self.pushing(high, low)
            }

            Instruction::Ori => self,
            Instruction::Rst6 => self,
            Instruction::Rm => self,
            Instruction::Sphl => self,
            Instruction::Jm => self,

            // 0xF3
            Instruction::Di => Self {
                interrupt_enabled: false,
                ..self
            },
            // 0xFB
            Instruction::Ei => Self {
                interrupt_enabled: true,
                ..self
            },

            Instruction::Cm => self,
            Instruction::Cpi => {
                let (new_state, byte) = self.reading_next_byte();

                let res = new_state.a.wrapping_sub(byte);
                let mut cc = new_state.cc.clone();
                cc.set(ConditionCodes::Z, res == 0);
                cc.set(ConditionCodes::S, (res & 0x80) == 0x80);
                cc.set(ConditionCodes::P, parity(res));
                cc.set(ConditionCodes::CY, new_state.a < byte);

                Self { cc, ..new_state }
            }
            Instruction::Rst7 => self,
        }
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
