use std::convert::TryFrom;
use std::path::Path;

use bitflags::bitflags;

use crate::disassembler::Instruction;

pub trait IOHandler {
    fn inp(&mut self, state: State8080, port: u8) -> State8080;
    fn out(&mut self, state: State8080, port: u8) -> State8080;
}

pub struct DummyIOHandler;

impl IOHandler for DummyIOHandler {
    fn inp(&mut self, state: State8080, _v: u8) -> State8080 {
        state
    }

    fn out(&mut self, state: State8080, _v: u8) -> State8080 {
        state
    }
}

bitflags! {
    #[repr(C)]
    pub struct ConditionCodes: u8 {
        const S = 0b10000000;
        const Z = 0b01000000;
        const AC = 0b00010000;
        const P = 0b00000100;
        const CY = 0b00000001;
        const PAD = 0b00000010;
    }
}

impl Default for ConditionCodes {
    fn default() -> Self {
        ConditionCodes::PAD
    }
}

pub struct BytePair {
    pub low: u8,
    pub high: u8,
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
#[derive(Default, Clone)]
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
    pub cc: ConditionCodes,
    pub interrupt_enabled: bool,
    pub memory: Vec<u8>,
    last_cycles: u8,
}

impl State8080 {
    pub fn new() -> Self {
        State8080 {
            memory: vec![0; 0x10000],
            ..Default::default()
        }
    }

    pub fn last_cycles(&self) -> u8 {
        self.last_cycles
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

    pub fn bc(&self) -> BytePair {
        BytePair {
            high: self.b,
            low: self.c,
        }
    }

    pub fn de(&self) -> BytePair {
        BytePair {
            high: self.d,
            low: self.e,
        }
    }

    pub fn hl(&self) -> BytePair {
        BytePair {
            high: self.h,
            low: self.l,
        }
    }

    pub fn s(&self) -> bool {
        self.cc.contains(ConditionCodes::S)
    }

    pub fn z(&self) -> bool {
        self.cc.contains(ConditionCodes::Z)
    }

    pub fn p(&self) -> bool {
        self.cc.contains(ConditionCodes::P)
    }

    pub fn cy(&self) -> bool {
        self.cc.contains(ConditionCodes::CY)
    }

    pub fn ac(&self) -> bool {
        self.cc.contains(ConditionCodes::AC)
    }

    fn memory_at_sp(&self) -> BytePair {
        let low_index = self.sp;
        let high_index = self.sp.wrapping_add(1);
        let low = self.memory[low_index as usize];
        let high = self.memory[high_index as usize];

        BytePair { high, low }
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
            cc: ConditionCodes::from_bits_truncate(bits | ConditionCodes::PAD.bits()),
            ..self
        }
    }

    fn setting_all_flags(self, value: u16, ac_check: u8) -> Self {
        self.setting_zspac_flags(value as u8, ac_check)
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

    fn setting_zsp_flags(self, value: u8) -> Self {
        self.setting_z_flag(value)
            .setting_s_flag(value)
            .setting_p_flag(value)
    }
    fn setting_zspac_flags(self, value: u8, ac_check: u8) -> Self {
        self.setting_zsp_flags(value).setting_ac_flag(ac_check)
    }

    fn clearing_ac(self) -> Self {
        self.setting_flag(ConditionCodes::AC, false)
    }

    fn clearing_cy(self) -> Self {
        self.setting_flag(ConditionCodes::CY, false)
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

    // TODO: Make this more pure-functional
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

    pub fn setting_a(self, a: u8) -> Self {
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
        let addr = pair.into();

        if condition {
            new_state.setting_pc(addr)
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

            new_state
                .setting_pc(addr)
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

    fn adding(self, rhs: u8, cy: bool) -> Self {
        let res_precise = self.a as u16 + rhs as u16 + cy as u16;
        let ac_check = (self.a & 0x0f) + (rhs & 0x0f) + cy as u8;
        let res = res_precise as u8;

        self.setting_a(res).setting_all_flags(res_precise, ac_check)
    }

    fn subtracting(self, rhs: u8, cy: bool) -> Self {
        let new_state = self.adding(!rhs, !cy);
        let new_cy = new_state.cy();

        new_state.setting_flag(ConditionCodes::CY, !new_cy)
    }

    fn ana(self, rhs: u8) -> Self {
        let lhs = self.a;
        let res = lhs & rhs;

        self.setting_a(res)
            .setting_zsp_flags(res)
            .setting_flag(ConditionCodes::AC, ((lhs | rhs) & 0x08) != 0)
            .clearing_cy()
    }

    fn xra(self, rhs: u8) -> Self {
        let res = self.a ^ rhs;

        self.setting_a(res)
            .setting_zsp_flags(res)
            .clearing_cy()
            .clearing_ac()
    }

    fn ora(self, rhs: u8) -> Self {
        let res = self.a | rhs;

        self.setting_a(res)
            .setting_zsp_flags(res)
            .clearing_cy()
            .clearing_ac()
    }

    fn cmp(self, rhs: u8) -> Self {
        let a = self.a;

        self.subtracting(rhs, false).setting_a(a)
    }

    fn rst(self, i: u16) -> Self {
        let pair = BytePair::from(self.pc);

        self.pushing(pair.high, pair.low).setting_pc(i * 8)
    }

    fn evaluating_instruction<I: IOHandler>(
        self,
        instruction: Instruction,
        io_handler: Option<&mut I>,
    ) -> Self {
        #[cfg(feature = "logging")]
        self.log_instruction(instruction.clone());

        // let state;
        let new_state = match instruction {
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

            // 0x40
            Instruction::MovBB => {
                let val = self.b;

                self.setting_b(val)
            }
            // 0x41
            Instruction::MovBC => {
                let val = self.c;

                self.setting_b(val)
            }
            // 0x42
            Instruction::MovBD => {
                let val = self.d;

                self.setting_b(val)
            }
            // 0x43
            Instruction::MovBE => {
                let val = self.e;

                self.setting_b(val)
            }
            // 0x44
            Instruction::MovBH => {
                let val = self.h;

                self.setting_b(val)
            }
            // 0x45
            Instruction::MovBL => {
                let val = self.l;

                self.setting_b(val)
            }
            // 0x46
            Instruction::MovBM => {
                let offset: u16 = self.hl().into();
                let val = self.memory[offset as usize];

                self.setting_b(val)
            }
            // 0x47
            Instruction::MovBA => {
                let val = self.a;

                self.setting_b(val)
            }

            // 0x48
            Instruction::MovCB => {
                let val = self.b;

                self.setting_c(val)
            }
            // 0x49
            Instruction::MovCC => {
                let val = self.c;

                self.setting_c(val)
            }
            // 0x4A
            Instruction::MovCD => {
                let val = self.d;

                self.setting_c(val)
            }
            // 0x4B
            Instruction::MovCE => {
                let val = self.e;

                self.setting_c(val)
            }
            // 0x4C
            Instruction::MovCH => {
                let val = self.h;

                self.setting_c(val)
            }
            // 0x4D
            Instruction::MovCL => {
                let val = self.l;

                self.setting_c(val)
            }
            // 0x4E
            Instruction::MovCM => {
                let offset: u16 = self.hl().into();
                let val = self.memory[offset as usize];

                self.setting_c(val)
            }
            // 0x4F
            Instruction::MovCA => {
                let val = self.a;

                self.setting_c(val)
            }

            // 0x50
            Instruction::MovDB => {
                let val = self.b;

                self.setting_d(val)
            }
            // 0x51
            Instruction::MovDC => {
                let val = self.c;

                self.setting_d(val)
            }
            // 0x52
            Instruction::MovDD => {
                let val = self.d;

                self.setting_d(val)
            }
            // 0x53
            Instruction::MovDE => {
                let val = self.e;

                self.setting_d(val)
            }
            // 0x54
            Instruction::MovDH => {
                let val = self.h;

                self.setting_d(val)
            }
            // 0x55
            Instruction::MovDL => {
                let val = self.l;

                self.setting_d(val)
            }
            // 0x56
            Instruction::MovDM => {
                let offset: u16 = self.hl().into();
                let val = self.memory[offset as usize];

                self.setting_d(val)
            }
            // 0x57
            Instruction::MovDA => {
                let val = self.a;

                self.setting_d(val)
            }

            // 0x58
            Instruction::MovEB => {
                let val = self.b;

                self.setting_e(val)
            }
            // 0x59
            Instruction::MovEC => {
                let val = self.c;

                self.setting_e(val)
            }
            // 0x5A
            Instruction::MovED => {
                let val = self.d;

                self.setting_e(val)
            }
            // 0x5B
            Instruction::MovEE => {
                let val = self.e;

                self.setting_e(val)
            }
            // 0x5C
            Instruction::MovEH => {
                let val = self.h;

                self.setting_e(val)
            }
            // 0x5D
            Instruction::MovEL => {
                let val = self.l;

                self.setting_e(val)
            }
            // 0x5E
            Instruction::MovEM => {
                let offset: u16 = self.hl().into();
                let val = self.memory[offset as usize];

                self.setting_e(val)
            }
            // 0x5F
            Instruction::MovEA => {
                let val = self.a;

                self.setting_e(val)
            }

            // 0x60
            Instruction::MovHB => {
                let val = self.b;

                self.setting_h(val)
            }
            // 0x61
            Instruction::MovHC => {
                let val = self.c;

                self.setting_h(val)
            }
            // 0x62
            Instruction::MovHD => {
                let val = self.d;

                self.setting_h(val)
            }
            // 0x63
            Instruction::MovHE => {
                let val = self.e;

                self.setting_h(val)
            }
            // 0x64
            Instruction::MovHH => {
                let val = self.h;

                self.setting_h(val)
            }
            // 0x65
            Instruction::MovHL => {
                let val = self.l;

                self.setting_h(val)
            }
            // 0x66
            Instruction::MovHM => {
                let offset: u16 = self.hl().into();
                let val = self.memory[offset as usize];

                self.setting_h(val)
            }
            // 0x67
            Instruction::MovHA => {
                let val = self.a;

                self.setting_h(val)
            }

            // 0x68
            Instruction::MovLB => {
                let val = self.b;

                self.setting_l(val)
            }
            // 0x69
            Instruction::MovLC => {
                let val = self.c;

                self.setting_l(val)
            }
            // 0x6A
            Instruction::MovLD => {
                let val = self.d;

                self.setting_l(val)
            }
            // 0x6B
            Instruction::MovLE => {
                let val = self.e;

                self.setting_l(val)
            }
            // 0x6C
            Instruction::MovLH => {
                let val = self.h;

                self.setting_l(val)
            }
            // 0x6D
            Instruction::MovLL => {
                let val = self.l;

                self.setting_l(val)
            }
            // 0x6E
            Instruction::MovLM => {
                let offset: u16 = self.hl().into();
                let val = self.memory[offset as usize];

                self.setting_l(val)
            }
            // 0x6F
            Instruction::MovLA => {
                let val = self.a;

                self.setting_l(val)
            }

            // 0x70
            Instruction::MovMB => {
                let offset = self.hl().into();
                let byte = self.b;
                self.setting_memory_at(byte, offset)
            }
            // 0x71
            Instruction::MovMC => {
                let offset = self.hl().into();
                let byte = self.c;
                self.setting_memory_at(byte, offset)
            }
            // 0x72
            Instruction::MovMD => {
                let offset = self.hl().into();
                let byte = self.d;
                self.setting_memory_at(byte, offset)
            }
            // 0x73
            Instruction::MovME => {
                let offset = self.hl().into();
                let byte = self.e;
                self.setting_memory_at(byte, offset)
            }
            // 0x74
            Instruction::MovMH => {
                let offset = self.hl().into();
                let byte = self.h;
                self.setting_memory_at(byte, offset)
            }
            // 0x75
            Instruction::MovML => {
                let offset = self.hl().into();
                let byte = self.l;
                self.setting_memory_at(byte, offset)
            }
            // 0x77
            Instruction::MovMA => {
                let offset = self.hl().into();
                let byte = self.a;
                self.setting_memory_at(byte, offset)
            }

            // 0x78
            Instruction::MovAB => {
                let val = self.b;

                self.setting_a(val)
            }
            // 0x79
            Instruction::MovAC => {
                let val = self.c;

                self.setting_a(val)
            }
            // 0x7A
            Instruction::MovAD => {
                let val = self.d;

                self.setting_a(val)
            }
            // 0x7B
            Instruction::MovAE => {
                let val = self.e;

                self.setting_a(val)
            }
            // 0x7C
            Instruction::MovAH => {
                let val = self.h;

                self.setting_a(val)
            }
            // 0x7D
            Instruction::MovAL => {
                let val = self.l;

                self.setting_a(val)
            }
            // 0x7E
            Instruction::MovAM => {
                let offset: u16 = self.hl().into();
                let val = self.memory[offset as usize];

                self.setting_a(val)
            }
            // 0x7F
            Instruction::MovAA => {
                let val = self.a;

                self.setting_a(val)
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
                let rhs = self.b;

                self.adding(rhs, false)
            }
            // 0x81
            Instruction::AddC => {
                let rhs = self.c;

                self.adding(rhs, false)
            }
            // 0x82
            Instruction::AddD => {
                let rhs = self.d;

                self.adding(rhs, false)
            }
            // 0x83
            Instruction::AddE => {
                let rhs = self.e;

                self.adding(rhs, false)
            }
            // 0x84
            Instruction::AddH => {
                let rhs = self.h;

                self.adding(rhs, false)
            }
            // 0x85
            Instruction::AddL => {
                let rhs = self.l;

                self.adding(rhs, false)
            }
            // 0x86
            Instruction::AddM => {
                let address: u16 = self.hl().into();
                let rhs = self.memory[address as usize];

                self.adding(rhs, false)
            }
            // 0x87
            Instruction::AddA => {
                let rhs = self.a;

                self.adding(rhs, false)
            }

            // 0xC6
            Instruction::Adi => {
                let (new_state, rhs) = self.reading_next_byte();

                new_state.adding(rhs, false)
            }

            // 0x88
            Instruction::AdcB => {
                let rhs = self.b;
                let cy = self.cy();

                self.adding(rhs, cy)
            }
            // 0x89
            Instruction::AdcC => {
                let rhs = self.c;
                let cy = self.cy();

                self.adding(rhs, cy)
            }
            // 0x8a
            Instruction::AdcD => {
                let rhs = self.d;
                let cy = self.cy();

                self.adding(rhs, cy)
            }
            // 0x8b
            Instruction::AdcE => {
                let rhs = self.e;
                let cy = self.cy();

                self.adding(rhs, cy)
            }
            // 0x8c
            Instruction::AdcH => {
                let rhs = self.h;
                let cy = self.cy();

                self.adding(rhs, cy)
            }
            // 0x8d
            Instruction::AdcL => {
                let rhs = self.l;
                let cy = self.cy();

                self.adding(rhs, cy)
            }
            // 0x8e
            Instruction::AdcM => {
                let address: u16 = self.hl().into();
                let rhs = self.memory[address as usize];
                let cy = self.cy();

                self.adding(rhs, cy)
            }
            // 0x8f
            Instruction::AdcA => {
                let rhs = self.a;
                let cy = self.cy();

                self.adding(rhs, cy)
            }

            // 0xCE
            Instruction::Aci => {
                let (new_state, rhs) = self.reading_next_byte();
                let cy = new_state.cy();

                new_state.adding(rhs, cy)
            }

            // 0x90
            Instruction::SubB => {
                let rhs = self.b;

                self.subtracting(rhs, false)
            }
            // 0x91
            Instruction::SubC => {
                let rhs = self.c;

                self.subtracting(rhs, false)
            }
            // 0x92
            Instruction::SubD => {
                let rhs = self.d;

                self.subtracting(rhs, false)
            }
            // 0x93
            Instruction::SubE => {
                let rhs = self.e;

                self.subtracting(rhs, false)
            }
            // 0x94
            Instruction::SubH => {
                let rhs = self.h;

                self.subtracting(rhs, false)
            }
            // 0x95
            Instruction::SubL => {
                let rhs = self.l;

                self.subtracting(rhs, false)
            }
            // 0x96
            Instruction::SubM => {
                let address: u16 = self.hl().into();
                let rhs = self.memory[address as usize];

                self.subtracting(rhs, false)
            }
            // 0x97
            Instruction::SubA => {
                let rhs = self.a;

                self.subtracting(rhs, false)
            }

            // 0xD6
            Instruction::Sui => {
                let (new_state, rhs) = self.reading_next_byte();

                new_state.subtracting(rhs, false)
            }

            // 0x98
            Instruction::SbbB => {
                let rhs = self.b;
                let cy = self.cy();

                self.subtracting(rhs, cy)
            }
            // 0x99
            Instruction::SbbC => {
                let rhs = self.c;
                let cy = self.cy();

                self.subtracting(rhs, cy)
            }
            // 0x9a
            Instruction::SbbD => {
                let rhs = self.d;
                let cy = self.cy();

                self.subtracting(rhs, cy)
            }
            // 0x9b
            Instruction::SbbE => {
                let rhs = self.e;
                let cy = self.cy();

                self.subtracting(rhs, cy)
            }
            // 0x9c
            Instruction::SbbH => {
                let rhs = self.h;
                let cy = self.cy();

                self.subtracting(rhs, cy)
            }
            // 0x9d
            Instruction::SbbL => {
                let rhs = self.l;
                let cy = self.cy();

                self.subtracting(rhs, cy)
            }
            // 0x9e
            Instruction::SbbM => {
                let address: u16 = self.hl().into();
                let rhs = self.memory[address as usize];
                let cy = self.cy();

                self.subtracting(rhs, cy)
            }
            // 0x9f
            Instruction::SbbA => {
                let rhs = self.a;
                let cy = self.cy();

                self.subtracting(rhs, cy)
            }

            // 0xDE
            Instruction::Sbi => {
                let (new_state, rhs) = self.reading_next_byte();
                let cy = new_state.cy();

                new_state.subtracting(rhs, cy)
            }

            // 0x04
            Instruction::InrB => {
                let res = self.b.wrapping_add(1);
                let ac_check = (self.b & 0x0f) + 1;

                self.setting_b(res).setting_zspac_flags(res, ac_check)
            }
            // 0x0C
            Instruction::InrC => {
                let res = self.c.wrapping_add(1);
                let ac_check = (self.c & 0x0f) + 1;

                self.setting_c(res).setting_zspac_flags(res, ac_check)
            }
            // 0x14
            Instruction::InrD => {
                let res = self.d.wrapping_add(1);
                let ac_check = (self.d & 0x0f) + 1;

                self.setting_d(res).setting_zspac_flags(res, ac_check)
            }
            // 0x1C
            Instruction::InrE => {
                let res = self.e.wrapping_add(1);
                let ac_check = (self.e & 0x0f) + 1;

                self.setting_e(res).setting_zspac_flags(res, ac_check)
            }
            // 0x24
            Instruction::InrH => {
                let res = self.h.wrapping_add(1);
                let ac_check = (self.h & 0x0f) + 1;

                self.setting_h(res).setting_zspac_flags(res, ac_check)
            }
            // 0x2C
            Instruction::InrL => {
                let res = self.l.wrapping_add(1);
                let ac_check = (self.l & 0x0f) + 1;

                self.setting_l(res).setting_zspac_flags(res, ac_check)
            }
            // 0x34
            Instruction::InrM => {
                let offset: u16 = self.hl().into();
                let res = self.memory[offset as usize].wrapping_add(1);
                let ac_check = (self.memory[offset as usize] & 0x0f) + 1;

                self.setting_memory_at(res, offset).setting_zspac_flags(res, ac_check)
            }
            // 0x3C
            Instruction::InrA => {
                let res = self.a.wrapping_add(1);
                let ac_check = (self.a & 0x0f) + 1;

                self.setting_a(res).setting_zspac_flags(res, ac_check)
            }

            // 0x05
            Instruction::DcrB => {
                let res = self.b.wrapping_sub(1);
                // +1 here is instead of !false as u8, by the idea of negating addition
                let ac_check = (self.b & 0x0f) + (!1 & 0x0f) + 1;

                self.setting_b(res).setting_zspac_flags(res, ac_check)
            }
            // 0x0D
            Instruction::DcrC => {
                let res = self.c.wrapping_sub(1);
                let ac_check = (self.c & 0x0f) + (!1 & 0x0f) + 1;

                self.setting_c(res).setting_zspac_flags(res, ac_check)
            }
            // 0x15
            Instruction::DcrD => {
                let res = self.d.wrapping_sub(1);
                let ac_check = (self.d & 0x0f) + (!1 & 0x0f) + 1;

                self.setting_d(res).setting_zspac_flags(res, ac_check)
            }
            // 0x1D
            Instruction::DcrE => {
                let res = self.e.wrapping_sub(1);
                let ac_check = (self.e & 0x0f) + (!1 & 0x0f) + 1;

                self.setting_e(res).setting_zspac_flags(res, ac_check)
            }
            // 0x25
            Instruction::DcrH => {
                let res = self.h.wrapping_sub(1);
                let ac_check = (self.h & 0x0f) + (!1 & 0x0f) + 1;

                self.setting_h(res).setting_zspac_flags(res, ac_check)
            }
            // 0x2D
            Instruction::DcrL => {
                let res = self.l.wrapping_sub(1);
                let ac_check = (self.l & 0x0f) + (!1 & 0x0f) + 1;

                self.setting_l(res).setting_zspac_flags(res, ac_check)
            }
            // 0x35
            Instruction::DcrM => {
                let offset: u16 = self.hl().into();
                let res = self.memory[offset as usize].wrapping_sub(1);
                let ac_check = (self.memory[offset as usize] & 0x0f) + (!1 & 0x0f) + 1;

                self.setting_memory_at(res, offset).setting_zspac_flags(res, ac_check)
            }
            // 0x3D
            Instruction::DcrA => {
                let res = self.a.wrapping_sub(1);
                let ac_check = (self.a & 0x0f) + (!1 & 0x0f) + 1;

                self.setting_a(res).setting_zspac_flags(res, ac_check)
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
                let res = hl.wrapping_add(bc);
                let res_pair = BytePair::from(res);

                self.setting_hl(res_pair).setting_flag(ConditionCodes::CY, hl > 0xffff - bc)
            }
            // 0x19
            Instruction::DadD => {
                let hl: u16 = self.hl().into();
                let de: u16 = self.de().into();
                let res = hl.wrapping_add(de);
                let res_pair = BytePair::from(res);

                self.setting_hl(res_pair).setting_flag(ConditionCodes::CY, hl > 0xffff - de)
            }
            // 0x29
            Instruction::DadH => {
                let hl: u16 = self.hl().into();
                let res = hl.wrapping_add(hl);
                let res_pair = BytePair::from(res);

                self.setting_hl(res_pair).setting_flag(ConditionCodes::CY, hl > 0xffff - hl)
            }
            // 0x39
            Instruction::DadSp => {
                let hl: u16 = self.hl().into();
                let sp = self.sp;
                let res = hl.wrapping_add(sp);
                let res_pair = BytePair::from(res);

                self.setting_hl(res_pair).setting_flag(ConditionCodes::CY, hl > 0xffff - sp)
            }

            // 0x27
            Instruction::Daa => {
                let mut correction = 0;
                let mut cy = self.cc.contains(ConditionCodes::CY);
                let ac = self.cc.contains(ConditionCodes::AC);

                let lsb = self.a & 0x0f;
                let msb = self.a & 0xf0;
                if ac || lsb > 9 {
                    correction += 6;
                }

                if cy || msb > 0x90 || (msb >= 0x90 && lsb > 9)  {
                    correction += 0x60;
                    cy = true;
                }

                // Not entirely sure about how flags should be set here
                self.adding(correction, false)
                    .setting_flag(ConditionCodes::CY, cy)
            }


            // Logical Group

            // 0xA0
            Instruction::AnaB => {
                let rhs = self.b;

                self.ana(rhs)
            }
            // 0xA1
            Instruction::AnaC => {
                let rhs = self.c;

                self.ana(rhs)
            }
            // 0xA2
            Instruction::AnaD => {
                let rhs = self.d;

                self.ana(rhs)
            }
            // 0xA3
            Instruction::AnaE => {
                let rhs = self.e;

                self.ana(rhs)
            }
            // 0xA4
            Instruction::AnaH => {
                let rhs = self.h;

                self.ana(rhs)
            }
            // 0xA5
            Instruction::AnaL => {
                let rhs = self.l;

                self.ana(rhs)
            }
            // 0xA6
            Instruction::AnaM => {
                let offset: u16 = self.hl().into();
                let rhs = self.memory[offset as usize];

                self.ana(rhs)
            }
            // 0xA7
            Instruction::AnaA => {
                let rhs = self.a;

                self.ana(rhs)
            }

            // 0xE6
            Instruction::Ani => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.ana(byte)
            }

            // 0xA8
            Instruction::XraB => {
                let rhs = self.b;

                self.xra(rhs)
            }
            // 0xA9
            Instruction::XraC => {
                let rhs = self.c;

                self.xra(rhs)
            }
            // 0xAA
            Instruction::XraD => {
                let rhs = self.d;

                self.xra(rhs)
            }
            // 0xAB
            Instruction::XraE => {
                let rhs = self.e;

                self.xra(rhs)
            }
            // 0xAC
            Instruction::XraH => {
                let rhs = self.h;

                self.xra(rhs)
            }
            // 0xAD
            Instruction::XraL => {
                let rhs = self.l;

                self.xra(rhs)
            }
            // 0xAE
            Instruction::XraM => {
                let offset: u16 = self.hl().into();
                let rhs = self.memory[offset as usize];

                self.xra(rhs)
            }
            // 0xAF
            Instruction::XraA => {
                let rhs = self.a;

                self.xra(rhs)
            }

            // 0xEE
            Instruction::Xri => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.xra(byte)
            }

            // 0xB0
            Instruction::OraB => {
                let rhs = self.b;

                self.ora(rhs)
            }
            // 0xB1
            Instruction::OraC => {
                let rhs = self.c;

                self.ora(rhs)
            }
            // 0xB2
            Instruction::OraD => {
                let rhs = self.d;

                self.ora(rhs)
            }
            // 0xB3
            Instruction::OraE => {
                let rhs = self.e;

                self.ora(rhs)
            }
            // 0xB4
            Instruction::OraH => {
                let rhs = self.h;

                self.ora(rhs)
            }
            // 0xB5
            Instruction::OraL => {
                let rhs = self.l;

                self.ora(rhs)
            }
            // 0xB6
            Instruction::OraM => {
                let offset: u16 = self.hl().into();
                let rhs = self.memory[offset as usize];

                self.ora(rhs)
            }
            // 0xB7
            Instruction::OraA => {
                let rhs = self.a;

                self.ora(rhs)
            }

            // 0xF6
            Instruction::Ori => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.ora(byte)
            }

            // 0xB8
            Instruction::CmpB => {
                let rhs = self.b;

                self.cmp(rhs)
            }
            // 0xB9
            Instruction::CmpC => {
                let rhs = self.c;

                self.cmp(rhs)
            }
            // 0xBA
            Instruction::CmpD => {
                let rhs = self.d;

                self.cmp(rhs)
            }
            // 0xBB
            Instruction::CmpE => {
                let rhs = self.e;

                self.cmp(rhs)
            }
            // 0xBC
            Instruction::CmpH => {
                let rhs = self.h;

                self.cmp(rhs)
            }
            // 0xBD
            Instruction::CmpL => {
                let rhs = self.l;

                self.cmp(rhs)
            }
            // 0xBE
            Instruction::CmpM => {
                let offset: u16 = self.hl().into();
                let rhs = self.memory[offset as usize];

                self.cmp(rhs)
            }
            // 0xBF
            Instruction::CmpA => {
                let rhs = self.a;

                self.cmp(rhs)
            }

            // 0xFE
            Instruction::Cpi => {
                let (new_state, byte) = self.reading_next_byte();

                new_state.cmp(byte)
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
                let a = self.cy() as u8 | (x << 1);

                self.setting_a(a)
                    .setting_flag(ConditionCodes::CY, (x & 0x80) == 0x80)
            }
            // 0x1F
            Instruction::Rar => {
                let x = self.a;
                let a = ((self.cy() as u8) << 7) | (x >> 1);

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
                let return_addr = new_state.pc;
                let return_pair = BytePair::from(return_addr);

                let high_mem_addr = new_state.sp.wrapping_sub(1);
                let low_mem_addr = new_state.sp.wrapping_sub(2);

                new_state.setting_pc(addr)
                    .setting_sp(low_mem_addr)
                    .setting_memory_at(return_pair.high, high_mem_addr)
                    .setting_memory_at(return_pair.low, low_mem_addr)
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

            Instruction::Rst0 => self.rst(0),
            Instruction::Rst1 => self.rst(1),
            Instruction::Rst2 => self.rst(2),
            Instruction::Rst3 => self.rst(3),
            Instruction::Rst4 => self.rst(4),
            Instruction::Rst5 => self.rst(5),
            Instruction::Rst6 => self.rst(6),
            Instruction::Rst7 => self.rst(7),

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
                let (new_state, b) = self.reading_next_byte();

                match io_handler {
                    Some(handler) => handler.out(new_state, b),
                    None => new_state,
                }
            }
            // 0xDB
            Instruction::In => {
                let (new_state, b) = self.reading_next_byte();

                match io_handler {
                    Some(handler) => handler.inp(new_state, b),
                    None => new_state,
                }
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

            // 0x76
            Instruction::Hlt => {
                println!("HLT called!");
                std::process::exit(0);
            }
        };

        Self {
            last_cycles: instruction.cycles(),
            ..new_state
        }
    }

    pub fn evaluating_next<I: IOHandler>(self, io_handler: Option<&mut I>) -> Self {
        let (mut state, op_code) = self.reading_next_byte();

        match Instruction::try_from(op_code) {
            Ok(instruction) => state = state.evaluating_instruction(instruction, io_handler),
            Err(_) => println!("Not an instruction: {:#04x}", op_code),
        }

        state
    }

    pub fn log_current_instruction(self) {
        let (state, op_code) = self.reading_next_byte();

        match Instruction::try_from(op_code) {
            Ok(instruction) => state.log_instruction(instruction.clone()),
            Err(_) => println!("Not an instruction: {:#04x}", op_code),
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
