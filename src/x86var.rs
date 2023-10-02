#![allow(unused)]

#[derive(Debug, PartialEq)]
pub struct X86VarProgram {
    pub(crate) blocks: Vec<(String, Block)>,
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub(crate) instrs: Vec<Instr>,
}

#[derive(Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Instr {
    Instr { cmd: Cmd, args: Vec<Arg> },
    Callq { lbl: String, arity: usize },
    Retq,
    Jmp { lbl: String },
}

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Addq,
    Subq,
    Negq,
    Movq,
    Pushq,
    Popq,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Arg {
    Imm { val: i64 },
    Reg { reg: Reg },
    Deref { reg: Reg, off: i64 },
    XVar { sym: String },
}

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum Reg {
    RSP,
    RBP,
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}
