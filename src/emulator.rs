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

    fn memory_at_sp(&self) -> BytePair {
        let low_index = self.sp;
        let high_index = self.sp.wrapping_add(1);
        let low = self.memory[low_index as usize];
        let high = self.memory[high_index as usize];

        BytePair {
            high,
            low,
        }
    }

    fn setting_memory_at_sp(self, pair: BytePair) -> Self {
        let low_index = self.sp;
        let high_index = self.sp.wrapping_add(1);
        
        self.setting_memory_at(pair.low, low_index)
            .setting_memory_at(pair.high, high_index)
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

    fn setting_raw_cc(self, bits: u8) -> Self {
        State8080 {
            cc: ConditionCodes::from_bits_truncate(bits),
            ..self
        }
    }

    fn setting_all_flags(self, value: u16) -> Self {
        self.setting_zspac_flags(value as u8)
            .setting_cy_flag(value)
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

    fn setting_cy_flag(self, value: u16) -> Self {
        self.setting_flag(ConditionCodes::CY, value > 0xff)
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

    fn setting_pc(self, pc: u16) -> Self {
        Self { pc, ..self }
    }



    fn jumping(self, condition: bool) -> Self {
        let (new_state, pair) = self.reading_next_pair();

        if condition {
            new_state.setting_pc(pair.into())
        } else {
            new_state
        }
    }

    fn calling(self, condition: bool) -> Self {
        let (new_state, pair) = self.reading_next_pair();

        if condition {
            let addr: u16 = pair.into();

            let return_addr = new_state.pc;
            let return_pair = BytePair::from(return_addr);

            let high_mem_addr = new_state.sp.wrapping_sub(1);
            let low_mem_addr = new_state.sp.wrapping_sub(2);

            new_state.setting_pc(addr)
                .setting_sp(low_mem_addr)
                .setting_memory_at(return_pair.high, high_mem_addr)
                .setting_memory_at(return_pair.low, low_mem_addr)
        } else {
            new_state
        }
    }

    fn popping(self) -> (Self, BytePair) {
        let low = self.memory[self.sp as usize];
        let high = self.memory[self.sp.wrapping_add(1) as usize];
        let popped = BytePair { low, high };
        let sp = self.sp.wrapping_add(2);
        
        (self.setting_sp(sp), popped)
    }

    fn returning(self, condition: bool) -> Self {
        if condition {
            let (new_state, popped) = self.popping();

            new_state.setting_pc(popped.into())
        } else {
            self
        }
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


            // Data Transfer Group

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
                let offset: u16 = new_state.hl().into();

                new_state.setting_memory_at(byte, offset)
            }
            // 0x3E
            Instruction::MviA => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.setting_a(byte)
            }
            
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

            // 0x02
            Instruction::StaxB => {
                let offset: u16 = self.bc().into();
                let val = self.a;

                self.setting_memory_at(val, offset)
            }
            // 0x12
            Instruction::StaxD => {
                let offset: u16 = self.de().into();
                let val = self.a;
                
                self.setting_memory_at(val, offset)
            }
            
            // 0x0A
            Instruction::LdaxB => {
                let offset: u16 = self.bc().into();
                let res = self.memory[offset as usize];
                
                self.setting_a(res)
            }
            // 0x1A
            Instruction::LdaxD => {
                let offset: u16 = self.de().into();
                let res = self.memory[offset as usize];

                self.setting_a(res)
            }

            // 0x32
            Instruction::Sta => {
                let (new_state, pair) = self.reading_next_pair();
                let offset: u16 = pair.into();
                let byte = new_state.a;
                new_state.setting_memory_at(byte, offset)
            }
            // 0x3A
            Instruction::Lda => {
                let (new_state, pair) = self.reading_next_pair();
                let offset: u16 = pair.into();
                let a = new_state.memory[offset as usize];

                new_state.setting_a(a)
            }

            // 0x22
            Instruction::Shld => {
                let (new_state, pair) = self.reading_next_pair();
                let offset: u16 = pair.into();
                let l = new_state.l;
                let h = new_state.h;

                new_state.setting_memory_at(l, offset)
                    .setting_memory_at(h, offset.wrapping_add(1))
            }
            // 0x2A
            Instruction::Lhld => {
                let (new_state, pair) = self.reading_next_pair();
                let offset: u16 = pair.into();
                let l = new_state.memory[offset as usize];
                let h = new_state.memory[offset as usize + 1];

                new_state.setting_l(l)
                    .setting_h(h)
            }

            // 0xEB
            Instruction::Xchg => Self {
                h: self.d,
                l: self.e,
                d: self.h,
                e: self.l,
                ..self
            },



            // Arithmetic Group

            // 0x80
            Instruction::AddB => {
                let res_precise = (self.a as u16).wrapping_add(self.b as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x81
            Instruction::AddC => {
                let res_precise = (self.a as u16).wrapping_add(self.c as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x82
            Instruction::AddD => {
                let res_precise = (self.a as u16).wrapping_add(self.d as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x83
            Instruction::AddE => {
                let res_precise = (self.a as u16).wrapping_add(self.e as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x84
            Instruction::AddH => {
                let res_precise = (self.a as u16).wrapping_add(self.h as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x85
            Instruction::AddL => {
                let res_precise = (self.a as u16).wrapping_add(self.l as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x86
            Instruction::AddM => {
                let address: u16 = self.hl().into();

                let res_precise =
                    (self.a as u16).wrapping_add(self.memory[address as usize] as u16);
                let res = (res_precise & 0xff) as u8;
                
                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x87
            Instruction::AddA => {
                let res_precise = (self.a as u16).wrapping_add(self.a as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }

            // 0xC6
            Instruction::Adi => {
                let (new_state, byte) = self.reading_next_byte();

                let res_precise = (new_state.a as u16).wrapping_add(byte as u16);
                let res = res_precise as u8;

                new_state.setting_a(res)
                    .setting_all_flags(res_precise)
            }

            // 0x88
            Instruction::AdcB => {
                let res_precise = (self.a as u16)
                    .wrapping_add(self.b as u16)
                    .wrapping_add(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x89
            Instruction::AdcC => {
                let res_precise = (self.a as u16)
                    .wrapping_add(self.c as u16)
                    .wrapping_add(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x8a
            Instruction::AdcD => {
                let res_precise = (self.a as u16)
                    .wrapping_add(self.d as u16)
                    .wrapping_add(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x8b
            Instruction::AdcE => {
                let res_precise = (self.a as u16)
                    .wrapping_add(self.e as u16)
                    .wrapping_add(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x8c
            Instruction::AdcH => {
                let res_precise = (self.a as u16)
                    .wrapping_add(self.h as u16)
                    .wrapping_add(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x8d
            Instruction::AdcL => {
                let res_precise = (self.a as u16)
                    .wrapping_add(self.l as u16)
                    .wrapping_add(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x8e
            Instruction::AdcM => {
                let address: u16 = self.hl().into();

                let res_precise = (self.a as u16)
                    .wrapping_add(self.memory[address as usize] as u16)
                    .wrapping_add(self.cc.contains(ConditionCodes::CY) as u16);
                let res = (res_precise & 0xff) as u8;
                
                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x8f
            Instruction::AdcA => {
                let res_precise = (self.a as u16)
                    .wrapping_add(self.a as u16)
                    .wrapping_add(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }

            // 0xCE
            Instruction::Aci => {
                let (new_state, byte) = self.reading_next_byte();

                let res_precise = (new_state.a as u16)
                    .wrapping_add(byte as u16)
                    .wrapping_add(new_state.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                new_state.setting_a(res)
                    .setting_all_flags(res_precise)
            }

            // 0x90
            Instruction::SubB => {
                let res_precise = (self.a as u16).wrapping_sub(self.b as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x91
            Instruction::SubC => {
                let res_precise = (self.a as u16).wrapping_sub(self.c as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x92
            Instruction::SubD => {
                let res_precise = (self.a as u16).wrapping_sub(self.d as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x93
            Instruction::SubE => {
                let res_precise = (self.a as u16).wrapping_sub(self.e as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x94
            Instruction::SubH => {
                let res_precise = (self.a as u16).wrapping_sub(self.h as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x95
            Instruction::SubL => {
                let res_precise = (self.a as u16).wrapping_sub(self.l as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x96
            Instruction::SubM => {
                let address: u16 = self.hl().into();

                let res_precise =
                    (self.a as u16).wrapping_sub(self.memory[address as usize] as u16);
                let res = (res_precise & 0xff) as u8;
                
                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x97
            Instruction::SubA => {
                let res_precise = (self.a as u16).wrapping_sub(self.a as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }

            // 0xD6
            Instruction::Sui => {
                let (new_state, byte) = self.reading_next_byte();

                let res_precise = (new_state.a as u16).wrapping_sub(byte as u16);
                let res = res_precise as u8;

                new_state.setting_a(res)
                    .setting_all_flags(res_precise)
            }

            // 0x98
            Instruction::SbbB => {
                let res_precise = (self.a as u16)
                    .wrapping_sub(self.b as u16)
                    .wrapping_sub(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x99
            Instruction::SbbC => {
                let res_precise = (self.a as u16)
                    .wrapping_sub(self.c as u16)
                    .wrapping_sub(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x9a
            Instruction::SbbD => {
                let res_precise = (self.a as u16)
                    .wrapping_sub(self.d as u16)
                    .wrapping_sub(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x9b
            Instruction::SbbE => {
                let res_precise = (self.a as u16)
                    .wrapping_sub(self.e as u16)
                    .wrapping_sub(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x9c
            Instruction::SbbH => {
                let res_precise = (self.a as u16)
                    .wrapping_sub(self.h as u16)
                    .wrapping_sub(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x9d
            Instruction::SbbL => {
                let res_precise = (self.a as u16)
                    .wrapping_sub(self.l as u16)
                    .wrapping_sub(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x9e
            Instruction::SbbM => {
                let address: u16 = self.hl().into();

                let res_precise = (self.a as u16)
                    .wrapping_sub(self.memory[address as usize] as u16)
                    .wrapping_sub(self.cc.contains(ConditionCodes::CY) as u16);
                let res = (res_precise & 0xff) as u8;
                
                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }
            // 0x9f
            Instruction::SbbA => {
                let res_precise = (self.a as u16)
                    .wrapping_sub(self.a as u16)
                    .wrapping_sub(self.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                self.setting_a(res)
                    .setting_all_flags(res_precise)
            }

            // 0xDE
            Instruction::Sbi => {
                let (new_state, byte) = self.reading_next_byte();

                let res_precise = (new_state.a as u16)
                    .wrapping_sub(byte as u16)
                    .wrapping_sub(new_state.cc.contains(ConditionCodes::CY) as u16);
                let res = res_precise as u8;

                new_state.setting_a(res)
                    .setting_all_flags(res_precise)
            }

            // 0x04
            Instruction::InrB => {
                let res = self.b.wrapping_add(1);

                self.setting_b(res).setting_zspac_flags(res)
            }
            // 0x0C
            Instruction::InrC => {
                let res = self.c.wrapping_add(1);

                self.setting_c(res).setting_zspac_flags(res)
            }
            // 0x14
            Instruction::InrD => {
                let res = self.d.wrapping_add(1);

                self.setting_d(res).setting_zspac_flags(res)
            }
            // 0x1C
            Instruction::InrE => {
                let res = self.e.wrapping_add(1);

                self.setting_e(res).setting_zspac_flags(res)
            }
            // 0x24
            Instruction::InrH => {
                let res = self.h.wrapping_add(1);

                self.setting_h(res).setting_zspac_flags(res)
            }
            // 0x2C
            Instruction::InrL => {
                let res = self.l.wrapping_add(1);

                self.setting_l(res).setting_zspac_flags(res)
            }
            // 0x34
            Instruction::InrM => {
                let offset: u16 = self.hl().into();
                let res = self.memory[offset as usize].wrapping_add(1);

                self.setting_memory_at(res, offset).setting_zspac_flags(res)
            }
            // 0x3C
            Instruction::InrA => {
                let res = self.a.wrapping_add(1);

                self.setting_a(res).setting_zspac_flags(res)
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

            // 0x03
            Instruction::InxB => {
                let c = self.c.wrapping_add(1);
                let mut b: Option<u8> = None;
                if c == 0 {
                    b = Some(self.b.wrapping_add(1));
                }

                let pair = BytePair { 
                    high: b.unwrap_or(self.b), 
                    low: c 
                };

                self.setting_bc(pair)
            }
            // 0x13
            Instruction::InxD => {
                let e = self.e.wrapping_add(1);
                let mut d: Option<u8> = None;
                if e == 0 {
                    d = Some(self.d.wrapping_add(1));
                }

                let pair = BytePair { 
                    high: d.unwrap_or(self.d), 
                    low: e 
                };

                self.setting_de(pair)
            }
            // 0x23
            Instruction::InxH => {
                let l = self.l.wrapping_add(1);
                let mut h: Option<u8> = None;
                if l == 0 {
                    h = Some(self.h.wrapping_add(1));
                }

                let pair = BytePair { 
                    high: h.unwrap_or(self.h), 
                    low: l 
                };

                self.setting_hl(pair)
            }
            // 0x33
            Instruction::InxSp => {
                let sp = self.sp.wrapping_add(1);
                
                self.setting_sp(sp)
            }

            // 0x0B
            Instruction::DcxB => {
                let c = self.c.wrapping_sub(1);
                let mut b: Option<u8> = None;
                if c == 0xff {
                    b = Some(self.b.wrapping_sub(1));
                }

                let pair = BytePair { 
                    high: b.unwrap_or(self.b), 
                    low: c 
                };

                self.setting_bc(pair)
            }
            // 0x1B
            Instruction::DcxD => {
                let e = self.e.wrapping_sub(1);
                let mut d: Option<u8> = None;
                if e == 0xff {
                    d = Some(self.d.wrapping_sub(1));
                }

                let pair = BytePair { 
                    high: d.unwrap_or(self.d), 
                    low: e
                };

                self.setting_de(pair)
            }
            // 0x2B
            Instruction::DcxH => {
                let l = self.l.wrapping_sub(1);
                let mut h: Option<u8> = None;
                if l == 0xff {
                    h = Some(self.h.wrapping_sub(1));
                }

                let pair = BytePair { 
                    high: h.unwrap_or(self.h), 
                    low: l
                };

                self.setting_hl(pair)
            }
            // 0x3B
            Instruction::DcxSp => {
                let sp = self.sp.wrapping_sub(1);
                
                self.setting_sp(sp)
            }

            // 0x09
            Instruction::DadB => {
                let hl: u16 = self.hl().into();
                let bc: u16 = self.bc().into();
                let res = (hl as u32).wrapping_add(bc as u32);
                let res_pair = BytePair::from(res as u16);

                self.setting_hl(res_pair).setting_flag(ConditionCodes::CY, (res & 0xff00) != 0)
            }
            // 0x19
            Instruction::DadD => {
                let hl: u16 = self.hl().into();
                let de: u16 = self.de().into();
                let res = (hl as u32).wrapping_add(de as u32);
                let res_pair = BytePair::from(res as u16);

                self.setting_hl(res_pair).setting_flag(ConditionCodes::CY, (res & 0xff00) != 0)
            }
            // 0x29
            Instruction::DadH => {
                let hl: u16 = self.hl().into();
                let res = (hl as u32).wrapping_add(hl as u32);
                let res_pair = BytePair::from(res as u16);

                self.setting_hl(res_pair).setting_flag(ConditionCodes::CY, (res & 0xff00) != 0)
            }
            // 0x39
            Instruction::DadSp => {
                let hl: u16 = self.hl().into();
                let sp = self.sp;
                let res = (hl as u32).wrapping_add(sp as u32);
                let res_pair = BytePair::from(res as u16);

                self.setting_hl(res_pair).setting_flag(ConditionCodes::CY, (res & 0xff00) != 0)
            }

            // 0x27
            Instruction::Daa => {
                let mut a: u16 = self.a as u16;

                if a & 0xf > 9 {
                    a += 6;
                }

                if a & 0xf0 > 0x90 {
                    a += 0x60;
                }
                
                // Not entirely sure about how flags should be set here
                self.setting_a(a as u8).setting_all_flags(a)
            }


            // Logical Group

            // 0xA0
            Instruction::AnaB => {
                let res = self.a & self.b;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xA1
            Instruction::AnaC => {
                let res = self.a & self.c;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xA2
            Instruction::AnaD => {
                let res = self.a & self.d;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xA3
            Instruction::AnaE => {
                let res = self.a & self.e;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xA4
            Instruction::AnaH => {
                let res = self.a & self.h;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xA5
            Instruction::AnaL => {
                let res = self.a & self.l;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xA6
            Instruction::AnaM => {
                let offset: u16 = self.hl().into();
                let m = self.memory[offset as usize];
                let res = self.a & m;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xA7
            Instruction::AnaA => {
                let res = self.a & self.a;

                self.setting_a(res).setting_all_flags(res as u16)
            }

            // 0xE6
            Instruction::Ani => {
                let (new_state, byte) = self.reading_next_byte();
                let res = new_state.a & byte;

                new_state.setting_a(res).setting_all_flags(res as u16)
            }

            // 0xA8
            Instruction::XraB => {
                let res = self.a ^ self.b;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xA9
            Instruction::XraC => {
                let res = self.a ^ self.c;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xAA
            Instruction::XraD => {
                let res = self.a ^ self.d;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xAB
            Instruction::XraE => {
                let res = self.a ^ self.e;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xAC
            Instruction::XraH => {
                let res = self.a ^ self.h;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xAD
            Instruction::XraL => {
                let res = self.a ^ self.l;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xAE
            Instruction::XraM => {
                let offset: u16 = self.hl().into();
                let m = self.memory[offset as usize];
                let res = self.a ^ m;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xAF
            Instruction::XraA => {
                let res = self.a ^ self.a;

                self.setting_a(res).setting_all_flags(res as u16)
            }

            // 0xEE
            Instruction::Xri => {
                let (new_state, byte) = self.reading_next_byte();
                let res = new_state.a ^ byte;

                new_state.setting_a(res).setting_all_flags(res as u16)
            }

            // 0xB0
            Instruction::OraB => {
                let res = self.a | self.b;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xB1
            Instruction::OraC => {
                let res = self.a | self.c;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xB2
            Instruction::OraD => {
                let res = self.a | self.d;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xB3
            Instruction::OraE => {
                let res = self.a | self.e;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xB4
            Instruction::OraH => {
                let res = self.a | self.h;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xB5
            Instruction::OraL => {
                let res = self.a | self.l;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xB6
            Instruction::OraM => {
                let offset: u16 = self.hl().into();
                let m = self.memory[offset as usize];
                let res = self.a | m;

                self.setting_a(res).setting_all_flags(res as u16)
            }
            // 0xB7
            Instruction::OraA => {
                let res = self.a | self.a;

                self.setting_a(res).setting_all_flags(res as u16)
            }

            // 0xF6
            Instruction::Ori => {
                let (new_state, byte) = self.reading_next_byte();
                let res = new_state.a | byte;

                new_state.setting_a(res).setting_all_flags(res as u16)
            }

            // 0xB8
            Instruction::CmpB => {
                let res = self.a.wrapping_sub(self.b);

                self.setting_all_flags(res as u16)
            }
            // 0xB9
            Instruction::CmpC => {
                let res = self.a.wrapping_sub(self.c);

                self.setting_all_flags(res as u16)
            }
            // 0xBA
            Instruction::CmpD => {
                let res = self.a.wrapping_sub(self.d);

                self.setting_all_flags(res as u16)
            }
            // 0xBB
            Instruction::CmpE => {
                let res = self.a.wrapping_sub(self.e);

                self.setting_all_flags(res as u16)
            }
            // 0xBC
            Instruction::CmpH => {
                let res = self.a.wrapping_sub(self.h);

                self.setting_all_flags(res as u16)
            }
            // 0xBD
            Instruction::CmpL => {
                let res = self.a.wrapping_sub(self.l);

                self.setting_all_flags(res as u16)
            }
            // 0xBE
            Instruction::CmpM => {
                let offset: u16 = self.hl().into();
                let m = self.memory[offset as usize];
                let res = self.a.wrapping_sub(m);

                self.setting_all_flags(res as u16)
            }
            // 0xBF
            Instruction::CmpA => {
                let res = self.a.wrapping_sub(self.a);

                self.setting_all_flags(res as u16)
            }

            // 0xFE
            Instruction::Cpi => {
                let (new_state, byte) = self.reading_next_byte();
                let res = new_state.a.wrapping_sub(byte);

                new_state.setting_a(res).setting_all_flags(res as u16)
            }

            // 0x07
            Instruction::Rlc => {
                let x = self.a;
                let a = ((x & 0x80) >> 7) | (x << 1);
                self.setting_a(a)
                    .setting_flag(ConditionCodes::CY, (x & 0x80) == 0x80)
            }
            // 0x0F
            Instruction::Rrc => {
                let x = self.a;
                let a = ((x & 1) << 7) | (x >> 1);

                self.setting_a(a)
                    .setting_flag(ConditionCodes::CY, (x & 1) == 1)
            }
            // 0x17
            Instruction::Ral => {
                let x = self.a;
                let carry_u8 = self.cc.contains(ConditionCodes::CY) as u8;
                let a = ((carry_u8 & 1) >> 7) | (x << 1);

                self.setting_a(a)
                    .setting_flag(ConditionCodes::CY, (x & 0x80) == 0x80)
            }
            // 0x1F
            Instruction::Rar => {
                let x = self.a;
                let carry_u8 = self.cc.contains(ConditionCodes::CY) as u8;
                let a = ((carry_u8 & 1) << 7) | (x >> 1);

                self.setting_a(a)
                    .setting_flag(ConditionCodes::CY, (x & 1) == 1)
            }
            
            // 0x2F
            Instruction::Cma => {
                let complement = !self.a;

                self.setting_a(complement)
            }
            
            // 0x3F
            Instruction::Cmc => {
                let complement = !self.cc.contains(ConditionCodes::CY);

                self.setting_flag(ConditionCodes::CY, complement)
            }

            // 0x37
            Instruction::Stc => self.setting_flag(ConditionCodes::CY, true),


            // Branch Group

            // 0xC3
            Instruction::Jmp => self.jumping(true),
            // 0xC2
            Instruction::Jnz => {
                let condition = !self.cc.contains(ConditionCodes::Z);
                
                self.jumping(condition)
            }
            // 0xCA
            Instruction::Jz => {
                let condition = self.cc.contains(ConditionCodes::Z);
                
                self.jumping(condition)
            }
            // 0xD2
            Instruction::Jnc => {
                let condition = !self.cc.contains(ConditionCodes::CY);
                
                self.jumping(condition)
            }
            // 0xDA
            Instruction::Jc => {
                let condition = self.cc.contains(ConditionCodes::CY);
                
                self.jumping(condition)
            }
            // 0xE2
            Instruction::Jpo => {
                let condition = !self.cc.contains(ConditionCodes::P);
                
                self.jumping(condition)
            }
            // 0xEA
            Instruction::Jpe => {
                let condition = self.cc.contains(ConditionCodes::P);
                
                self.jumping(condition)
            }
            // 0xF2
            Instruction::Jp => {
                let condition = !self.cc.contains(ConditionCodes::S);
                
                self.jumping(condition)
            }
            // 0xFA
            Instruction::Jm => {
                let condition = self.cc.contains(ConditionCodes::S);
                
                self.jumping(condition)
            }

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
                        print!("{}", new_state.e as char);
                    }

                    new_state
                } else if cfg!(feature = "diagsupport") && addr == 0 {
                    panic!("Diag hit call 0");
                } else {
                    let return_addr = new_state.pc;
                    let return_pair = BytePair::from(return_addr);

                    let high_mem_addr = new_state.sp.wrapping_sub(1);
                    let low_mem_addr = new_state.sp.wrapping_sub(2);

                    new_state.setting_pc(addr)
                        .setting_sp(low_mem_addr)
                        .setting_memory_at(return_pair.high, high_mem_addr)
                        .setting_memory_at(return_pair.low, low_mem_addr)
                }
            }
            // 0xC4
            Instruction::Cnz =>  {
                let condition = !self.cc.contains(ConditionCodes::Z);
                
                self.calling(condition)
            }
            // 0xCC
            Instruction::Cz => {
                let condition = self.cc.contains(ConditionCodes::Z);
                
                self.calling(condition)
            }
            // 0xD4
            Instruction::Cnc => {
                let condition = !self.cc.contains(ConditionCodes::CY);
                
                self.calling(condition)
            }
            // 0xDC
            Instruction::Cc => {
                let condition = self.cc.contains(ConditionCodes::CY);
                
                self.calling(condition)
            }
            // 0xE4
            Instruction::Cpo => {
                let condition = !self.cc.contains(ConditionCodes::P);
                
                self.calling(condition)
            }
            // 0xEC
            Instruction::Cpe => {
                let condition = self.cc.contains(ConditionCodes::P);
                
                self.calling(condition)
            }
            // 0xF4
            Instruction::Cp => {
                let condition = !self.cc.contains(ConditionCodes::S);
                
                self.calling(condition)
            }
            // 0xFC
            Instruction::Cm => {
                let condition = self.cc.contains(ConditionCodes::S);
                
                self.calling(condition)
            }

            // 0xC9
            Instruction::Ret => self.returning(true),

            Instruction::Rnz => {
                let condition = !self.cc.contains(ConditionCodes::Z);
                
                self.returning(condition)
            }

            Instruction::Rz => {
                let condition = self.cc.contains(ConditionCodes::Z);
                
                self.returning(condition)
            }

            Instruction::Rnc => {
                let condition = !self.cc.contains(ConditionCodes::CY);
                
                self.returning(condition)
            }

            Instruction::Rc => {
                let condition = self.cc.contains(ConditionCodes::CY);
                
                self.returning(condition)
            }

            Instruction::Rpo => {
                let condition = !self.cc.contains(ConditionCodes::P);
                
                self.returning(condition)
            }

            Instruction::Rpe => {
                let condition = self.cc.contains(ConditionCodes::P);
                
                self.returning(condition)
            }

            Instruction::Rp => {
                let condition = !self.cc.contains(ConditionCodes::S);
                
                self.returning(condition)
            }

            Instruction::Rm => {
                let condition = self.cc.contains(ConditionCodes::S);
                
                self.returning(condition)
            }

            Instruction::Pchl => {
                let res = self.hl().into();

                self.setting_pc(res)
            }

            Instruction::Rst0 => self,
            Instruction::Rst1 => self,
            Instruction::Rst2 => self,
            Instruction::Rst3 => self,
            Instruction::Rst4 => self,
            Instruction::Rst5 => self,
            Instruction::Rst6 => self,
            Instruction::Rst7 => self,

            // Stack Group

            // 0xC5
            Instruction::PushB => {
                let (high, low) = (self.b, self.c);
                self.pushing(high, low)
            }
            // 0xD5
            Instruction::PushD => {
                let (high, low) = (self.d, self.e);
                self.pushing(high, low)
            }
            // 0xE5
            Instruction::PushH => {
                let (high, low) = (self.h, self.l);
                self.pushing(high, low)
            }
            // 0xF5
            Instruction::PushPsw => {
                let (high, low) = (self.a, self.cc.bits);

                self.pushing(high, low)
            }

            // 0xC1
            Instruction::PopB => {
                let (new_state, popped) = self.popping();

                new_state.setting_bc(popped.into())
            }
            // 0xD1
            Instruction::PopD => {
                let (new_state, popped) = self.popping();

                new_state.setting_de(popped.into())
            }
            // 0xE1
            Instruction::PopH => {
                let (new_state, popped) = self.popping();

                new_state.setting_hl(popped.into())
            }
            // 0xF1
            Instruction::PopPsw => {
                let (new_state, popped) = self.popping();

                new_state.setting_a(popped.high).setting_raw_cc(popped.low)
            }

            // 0xE3
            Instruction::Xthl => {
                let hl_pair = self.hl();
                let sp_pair = self.memory_at_sp();

                self.setting_hl(sp_pair).setting_memory_at_sp(hl_pair)
            }

            // 0xF9
            Instruction::Sphl => {
                let hl = self.hl();

                self.setting_sp(hl.into())
            }

            // 0xD3
            Instruction::Out => {
                let (new_state, _b) = self.reading_next_byte();
                new_state
                // (state.output_handler.0)(b);
            }
            // 0xDB
            Instruction::In => {
                let (new_state, _b) = self.reading_next_byte();
                new_state
                // (state.input_handler.0)(b);
            }

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
