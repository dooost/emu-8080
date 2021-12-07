macro_rules! enum_try_from_u8 {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::TryFrom<u8> for $name {
            type Error = ();

            fn try_from(v: u8) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as u8 => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
    }
}

enum_try_from_u8! {
    #[derive(Debug, Clone)]
    #[repr(u8)]
    pub enum Instruction {
        Nop = 0x00,
        LxiB,
        StaxB,
        InxB,
        InrB,
        DcrB,
        MviB,
        Rlc,
        Nop1,
        DadB,
        LdaxB,
        DcxB,
        InrC,
        DcrC,
        MviC,
        Rrc,
        Nop2,
        LxiD,
        StaxD,
        InxD,
        InrD,
        DcrD,
        MviD,
        Ral,
        Nop3,
        DadD,
        LdaxD,
        DcxD,
        InrE,
        DcrE,
        MviE,
        Rar,
        Nop4,
        LxiH,
        Shld,
        InxH,
        InrH,
        DcrH,
        MviH,
        Daa,
        Nop5,
        DadH,
        Lhld,
        DcxH,
        InrL,
        DcrL,
        MviL,
        Cma,
        Nop6,
        LxiSp,
        Sta,
        InxSp,
        InrM,
        DcrM,
        MviM,
        Stc,
        Nop7,
        DadSp,
        Lda,
        DcxSp,
        InrA,
        DcrA,
        MviA,
        Cmc,
        MovBB,
        MovBC,
        MovBD,
        MovBE,
        MovBH,
        MovBL,
        MovBM,
        MovBA,
        MovCB,
        MovCC,
        MovCD,
        MovCE,
        MovCH,
        MovCL,
        MovCM,
        MovCA,
        MovDB,
        MovDC,
        MovDD,
        MovDE,
        MovDH,
        MovDL,
        MovDM,
        MovDA,
        MovEB,
        MovEC,
        MovED,
        MovEE,
        MovEH,
        MovEL,
        MovEM,
        MovEA,
        MovHB,
        MovHC,
        MovHD,
        MovHE,
        MovHH,
        MovHL,
        MovHM,
        MovHA,
        MovLB,
        MovLC,
        MovLD,
        MovLE,
        MovLH,
        MovLL,
        MovLM,
        MovLA,
        MovMB,
        MovMC,
        MovMD,
        MovME,
        MovMH,
        MovML,
        Hlt,
        MovMA,
        MovAB,
        MovAC,
        MovAD,
        MovAE,
        MovAH,
        MovAL,
        MovAM,
        MovAA,
        AddB,
        AddC,
        AddD,
        AddE,
        AddH,
        AddL,
        AddM,
        AddA,
        AdcB,
        AdcC,
        AdcD,
        AdcE,
        AdcH,
        AdcL,
        AdcM,
        AdcA,
        SubB,
        SubC,
        SubD,
        SubE,
        SubH,
        SubL,
        SubM,
        SubA,
        SbbB,
        SbbC,
        SbbD,
        SbbE,
        SbbH,
        SbbL,
        SbbM,
        SbbA,
        AnaB,
        AnaC,
        AnaD,
        AnaE,
        AnaH,
        AnaL,
        AnaM,
        AnaA,
        XraB,
        XraC,
        XraD,
        XraE,
        XraH,
        XraL,
        XraM,
        XraA,
        OraB,
        OraC,
        OraD,
        OraE,
        OraH,
        OraL,
        OraM,
        OraA,
        CmpB,
        CmpC,
        CmpD,
        CmpE,
        CmpH,
        CmpL,
        CmpM,
        CmpA,
        Rnz,
        PopB,
        Jnz,
        Jmp,
        Cnz,
        PushB,
        Adi,
        Rst0,
        Rz,
        Ret,
        Jz,
        Cz = 0xcc,
        Call,
        Aci,
        Rst1,
        Rnc,
        PopD,
        Jnc,
        Out,
        Cnc,
        PushD,
        Sui,
        Rst2,
        Rc,
        Jc = 0xda,
        In,
        Cc,
        Sbi = 0xde,
        Rst3,
        Rpo,
        PopH,
        Jpo,
        Xthl,
        Cpo,
        PushH,
        Ani,
        Rst4,
        Rpe,
        Pchl,
        Jpe,
        Xchg,
        Cpe,
        Xri = 0xee,
        Rst5,
        Rp,
        PopPsw,
        Jp,
        Di,
        Cp,
        PushPsw,
        Ori,
        Rst6,
        Rm,
        Sphl,
        Jm,
        Ei,
        Cm,
        Cpi = 0xfe,
        Rst7,
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Instruction {
    pub fn size(&self) -> u8 {
        match self {
            Instruction::LxiB
            | Instruction::LxiD
            | Instruction::LxiH
            | Instruction::Shld
            | Instruction::Lhld
            | Instruction::LxiSp
            | Instruction::Sta
            | Instruction::Lda
            | Instruction::Jnz
            | Instruction::Jmp
            | Instruction::Cnz
            | Instruction::Jz
            | Instruction::Cz
            | Instruction::Call
            | Instruction::Jnc
            | Instruction::Cnc
            | Instruction::Jc
            | Instruction::Cc
            | Instruction::Jpo
            | Instruction::Cpo
            | Instruction::Jpe
            | Instruction::Cpe
            | Instruction::Jp
            | Instruction::Cp
            | Instruction::Jm
            | Instruction::Cm => 3,

            Instruction::MviB
            | Instruction::MviC
            | Instruction::MviD
            | Instruction::MviE
            | Instruction::MviH
            | Instruction::MviL
            | Instruction::MviM
            | Instruction::MviA
            | Instruction::Adi
            | Instruction::Aci
            | Instruction::Out
            | Instruction::Sui
            | Instruction::In
            | Instruction::Sbi
            | Instruction::Ani
            | Instruction::Xri
            | Instruction::Ori
            | Instruction::Cpi => 2,

            _ => 1,
        }
    }

    pub fn cycles(&self) -> u32 {
        match self {
            Instruction::MovBB
            | Instruction::MovBC
            | Instruction::MovBD
            | Instruction::MovBE
            | Instruction::MovBH
            | Instruction::MovBL
            | Instruction::MovBA
            | Instruction::MovCB
            | Instruction::MovCC
            | Instruction::MovCD
            | Instruction::MovCE
            | Instruction::MovCH
            | Instruction::MovCL
            | Instruction::MovCA
            | Instruction::MovDB
            | Instruction::MovDC
            | Instruction::MovDD
            | Instruction::MovDE
            | Instruction::MovDH
            | Instruction::MovDL
            | Instruction::MovDA
            | Instruction::MovEB
            | Instruction::MovEC
            | Instruction::MovED
            | Instruction::MovEE
            | Instruction::MovEH
            | Instruction::MovEL
            | Instruction::MovEA
            | Instruction::MovHB
            | Instruction::MovHC
            | Instruction::MovHD
            | Instruction::MovHE
            | Instruction::MovHH
            | Instruction::MovHL
            | Instruction::MovHA
            | Instruction::MovLB
            | Instruction::MovLC
            | Instruction::MovLD
            | Instruction::MovLE
            | Instruction::MovLH
            | Instruction::MovLL
            | Instruction::MovLA
            | Instruction::MovAB
            | Instruction::MovAC
            | Instruction::MovAD
            | Instruction::MovAE
            | Instruction::MovAH
            | Instruction::MovAL
            | Instruction::MovAA => 5,

            Instruction::MovMB
            | Instruction::MovMC
            | Instruction::MovMD
            | Instruction::MovME
            | Instruction::MovMH
            | Instruction::MovML
            | Instruction::MovMA
            | Instruction::MovBM
            | Instruction::MovCM
            | Instruction::MovDM
            | Instruction::MovEM
            | Instruction::MovHM
            | Instruction::MovLM
            | Instruction::MovAM => 7,

            Instruction::MviB
            | Instruction::MviC
            | Instruction::MviD
            | Instruction::MviE
            | Instruction::MviH
            | Instruction::MviL
            | Instruction::MviA => 7,

            Instruction::MviM => 10,

            Instruction::LxiB | Instruction::LxiD | Instruction::LxiH | Instruction::LxiSp => 10,

            Instruction::Lda | Instruction::Sta => 13,

            Instruction::Lhld | Instruction::Shld => 16,

            Instruction::LdaxB | Instruction::LdaxD | Instruction::StaxB | Instruction::StaxD => 7,

            Instruction::Xchg => 4,

            Instruction::AddB
            | Instruction::AddC
            | Instruction::AddD
            | Instruction::AddE
            | Instruction::AddH
            | Instruction::AddL
            | Instruction::AddA
            | Instruction::AdcB
            | Instruction::AdcC
            | Instruction::AdcD
            | Instruction::AdcE
            | Instruction::AdcH
            | Instruction::AdcL
            | Instruction::AdcA
            | Instruction::SubB
            | Instruction::SubC
            | Instruction::SubD
            | Instruction::SubE
            | Instruction::SubH
            | Instruction::SubL
            | Instruction::SubA
            | Instruction::SbbB
            | Instruction::SbbC
            | Instruction::SbbD
            | Instruction::SbbE
            | Instruction::SbbH
            | Instruction::SbbL
            | Instruction::SbbA => 4,

            Instruction::AddM
            | Instruction::AdcM
            | Instruction::SubM
            | Instruction::SbbM
            | Instruction::Adi
            | Instruction::Aci
            | Instruction::Sui
            | Instruction::Sbi => 7,

            Instruction::InrB
            | Instruction::InrC
            | Instruction::InrD
            | Instruction::InrE
            | Instruction::InrH
            | Instruction::InrL
            | Instruction::InrA
            | Instruction::DcrB
            | Instruction::DcrC
            | Instruction::DcrD
            | Instruction::DcrE
            | Instruction::DcrH
            | Instruction::DcrL
            | Instruction::DcrA => 5,

            Instruction::InrM | Instruction::DcrM => 10,

            Instruction::InxB
            | Instruction::InxD
            | Instruction::InxH
            | Instruction::InxSp
            | Instruction::DcxB
            | Instruction::DcxD
            | Instruction::DcxH
            | Instruction::DcxSp => 5,

            Instruction::DadB | Instruction::DadD | Instruction::DadH | Instruction::DadSp => 10,

            Instruction::Daa => 4,

            Instruction::AnaB
            | Instruction::AnaC
            | Instruction::AnaD
            | Instruction::AnaE
            | Instruction::AnaH
            | Instruction::AnaL
            | Instruction::AnaA
            | Instruction::XraB
            | Instruction::XraC
            | Instruction::XraD
            | Instruction::XraE
            | Instruction::XraH
            | Instruction::XraL
            | Instruction::XraA
            | Instruction::OraB
            | Instruction::OraC
            | Instruction::OraD
            | Instruction::OraE
            | Instruction::OraH
            | Instruction::OraL
            | Instruction::OraA
            | Instruction::CmpB
            | Instruction::CmpC
            | Instruction::CmpD
            | Instruction::CmpE
            | Instruction::CmpH
            | Instruction::CmpL
            | Instruction::CmpA => 4,

            Instruction::AnaM
            | Instruction::XraM
            | Instruction::OraM
            | Instruction::CmpM
            | Instruction::Ani
            | Instruction::Xri
            | Instruction::Ori
            | Instruction::Cpi => 7,

            Instruction::Rlc | Instruction::Rrc | Instruction::Ral | Instruction::Rar => 4,

            Instruction::Cma | Instruction::Cmc | Instruction::Stc => 4,

            Instruction::Jmp
            | Instruction::Jnz
            | Instruction::Jz
            | Instruction::Jnc
            | Instruction::Jc
            | Instruction::Jpo
            | Instruction::Jpe
            | Instruction::Jp
            | Instruction::Jm => 10,

            Instruction::Cnz
            | Instruction::Cz
            | Instruction::Call
            | Instruction::Cnc
            | Instruction::Cc
            | Instruction::Cpo
            | Instruction::Cpe
            | Instruction::Cp
            | Instruction::Cm => 17,

            Instruction::Ret => 10,

            Instruction::Rnz
            | Instruction::Rz
            | Instruction::Rnc
            | Instruction::Rc
            | Instruction::Rpo
            | Instruction::Rpe
            | Instruction::Rp
            | Instruction::Rm => 11,

            Instruction::Rst0
            | Instruction::Rst1
            | Instruction::Rst2
            | Instruction::Rst3
            | Instruction::Rst4
            | Instruction::Rst5
            | Instruction::Rst6
            | Instruction::Rst7 => 11,

            Instruction::Pchl => 5,

            Instruction::PushB | Instruction::PushD | Instruction::PushH | Instruction::PushPsw => {
                11
            }

            Instruction::PopB | Instruction::PopD | Instruction::PopH | Instruction::PopPsw => 10,

            Instruction::Xthl => 18,

            Instruction::Sphl => 5,

            Instruction::In | Instruction::Out => 10,

            Instruction::Ei | Instruction::Di => 4,

            Instruction::Hlt => 7,

            Instruction::Nop
            | Instruction::Nop1
            | Instruction::Nop2
            | Instruction::Nop3
            | Instruction::Nop4
            | Instruction::Nop5
            | Instruction::Nop6
            | Instruction::Nop7 => 4,
        }
    }
}
