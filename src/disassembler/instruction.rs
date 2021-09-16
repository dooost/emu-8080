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
    #[derive(Debug)]
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
        DadB = 0x09,
        LdaxB,
        DcxB,
        InrC,
        DcrC,
        MviC,
        Rrc,
        LxiD = 0x11,
        StaxD,
        InxD,
        InrD,
        DcrD,
        MviD,
        Ral,
        DadD = 0x19,
        LdaxD,
        DcxD,
        InrE,
        DcrE,
        MviE,
        Rar,
        LxiH = 0x21,
        Shld,
        InxH,
        InrH,
        DcrH,
        MviH,
        Daa,
        DadH = 0x29,
        Lhld,
        DcxH,
        InrL,
        DcrL,
        MviL,
        Cma,
        LxiSp = 0x31,
        Sta,
        InxSp,
        InrM,
        DcrM,
        MviM,
        Stc,
        DadSp = 0x39,
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
}
