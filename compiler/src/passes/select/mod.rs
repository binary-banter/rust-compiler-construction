mod display;
pub mod macros;
pub mod select;

use crate::utils::unique_sym::UniqueSym;
use derive_more::Display;
use functor_derive::Functor;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Display;

pub struct X86Selected<'p> {
    pub fns: HashMap<UniqueSym<'p>, FunSelected<'p>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug)]
pub struct FunSelected<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, VarArg<UniqueSym<'p>>>>,
    pub entry: UniqueSym<'p>,
    pub exit: UniqueSym<'p>,
}

#[derive(Debug, Clone, Display, Functor)]
#[display(fmt = "\t{}", r#"instrs.iter().format("\n\t")"#)]
pub struct Block<'p, A: Display> {
    pub instrs: Vec<Instr<A, UniqueSym<'p>>>,
}

pub type InstrSelected<'p> = Instr<VarArg<UniqueSym<'p>>, UniqueSym<'p>>;

#[derive(Debug, Copy, Clone, PartialEq, Display)]
pub enum Cnd {
    Above,
    AboveOrEqual,
    Below,
    BelowOrEqual,
    Carry,
    EQ,
    GT,
    GE,
    LT,
    LE,
    NotCarry,
    NE,
    NotOverflow,
    NotSign,
    Overflow,
    ParityEven,
    ParityOdd,
    Sign,
}

#[derive(Debug, Clone, PartialEq, Display, Functor)]
pub enum Instr<Arg: Display, IdentVars: Display> {
    /// Add. https://www.felixcloutier.com/x86/add.
    #[display(fmt = "add\t{src} {dst}")]
    Add { src: Arg, dst: Arg },
    /// Subtract. https://www.felixcloutier.com/x86/sub.
    #[display(fmt = "sub\t{src} {dst}")]
    Sub { src: Arg, dst: Arg },
    /// Unsigned Divide. https://www.felixcloutier.com/x86/div.
    #[display(fmt = "div\t{divisor}")]
    Div { divisor: Arg },
    /// Signed Divide. https://www.felixcloutier.com/x86/idiv.
    #[display(fmt = "div\t{divisor}")]
    IDiv { divisor: Arg },
    /// Unsigned Multiply. https://www.felixcloutier.com/x86/mul.
    #[display(fmt = "mul\t{src}")]
    Mul { src: Arg },
    /// Signed Multiply. https://www.felixcloutier.com/x86/imul.
    #[display(fmt = "mul\t{src}")]
    IMul { src: Arg },
    /// Two's Complement Negation. https://www.felixcloutier.com/x86/neg.
    #[display(fmt = "neg\t{dst}")]
    Neg { dst: Arg },
    /// Move. https://www.felixcloutier.com/x86/mov.
    #[display(fmt = "mov\t{src} {dst}")]
    Mov { src: Arg, dst: Arg },
    /// Move with sign extension. https://www.felixcloutier.com/x86/movsx:movsxd.
    #[display(fmt = "mov\t{src} {dst}")]
    MovSX { src: Arg, dst: Arg },
    #[display(fmt = "push\t{src}")]
    /// Push Word, Doubleword, or Quadword Onto the Stack. https://www.felixcloutier.com/x86/push.
    Push { src: Arg },
    /// Pop a Value From the Stack. https://www.felixcloutier.com/x86/pop.
    #[display(fmt = "pop\t{dst}")]
    Pop { dst: Arg },
    /// Return From Procedure. https://www.felixcloutier.com/x86/ret.
    #[display(fmt = "ret")]
    Ret,
    /// Fast System Call. https://www.felixcloutier.com/x86/syscall.
    #[display(fmt = "syscall\t{arity}")]
    Syscall { arity: usize },
    /// Compare Two Operands. https://www.felixcloutier.com/x86/cmp.
    #[display(fmt = "cmp\t{src} {dst}")]
    Cmp { src: Arg, dst: Arg },
    /// Jump. https://www.felixcloutier.com/x86/jmp.
    #[display(fmt = "jmp\t{lbl}")]
    Jmp { lbl: IdentVars },
    /// Jump if Condition Is Met. https://www.felixcloutier.com/x86/jcc.
    #[display(fmt = "jcc\t{cnd} {lbl}")]
    Jcc { lbl: IdentVars, cnd: Cnd },
    /// Logical AND. https://www.felixcloutier.com/x86/and.
    #[display(fmt = "and {src} {dst}")]
    And { src: Arg, dst: Arg },
    /// Logical Inclusive OR. https://www.felixcloutier.com/x86/or.
    #[display(fmt = "orq {src} {dst}")]
    Or { src: Arg, dst: Arg },
    /// Logical Exclusive OR. https://www.felixcloutier.com/x86/xor.
    #[display(fmt = "xor\t{src} {dst}")]
    Xor { src: Arg, dst: Arg },
    #[display(fmt = "not\t{dst}")]
    /// One's Complement Negation. https://www.felixcloutier.com/x86/not.
    Not { dst: Arg },
    /// Set Byte On Condition. https://www.felixcloutier.com/x86/setcc
    #[display(fmt = "setcc\t{cnd}")]
    Setcc { cnd: Cnd },
    /// Load label as address by using a [`Mov`](Instr::Mov).
    #[display(fmt = "loadlbl\t{lbl} {dst}")]
    LoadLbl { lbl: IdentVars, dst: Arg },
    /// Call Procedure. https://www.felixcloutier.com/x86/call.
    #[display(fmt = "call_direct\t{lbl} {arity}")]
    CallDirect { lbl: IdentVars, arity: usize },
    /// Call indirect. https://www.felixcloutier.com/x86/call.
    #[display(fmt = "call_indirect\t{src} {arity}")]
    CallIndirect { src: Arg, arity: usize },
}

#[derive(Debug, PartialEq, Clone, Display, Functor)]
pub enum VarArg<IdentVars: Display> {
    #[display(fmt = "${_0}")]
    Imm(i32),
    #[display(fmt = "%{_0}")]
    Reg(Reg),
    #[display(fmt = "[%{reg} + ${off}]")]
    Deref { reg: Reg, off: i64 },
    #[display(fmt = "{_0}")]
    XVar(IdentVars),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Size {
    Bit8,
    Bit16,
    Bit32,
    Bit64,
}

pub const CALLER_SAVED: [Reg; 9] = [
    Reg::RAX,
    Reg::RCX,
    Reg::RDX,
    Reg::RSI,
    Reg::RDI,
    Reg::R8,
    Reg::R9,
    Reg::R10,
    Reg::R11,
];
pub const CALLEE_SAVED: [Reg; 7] = [
    Reg::RSP,
    Reg::RBP,
    Reg::RBX,
    Reg::R12,
    Reg::R13,
    Reg::R14,
    Reg::R15,
];
pub const CALLEE_SAVED_NO_STACK: [Reg; 5] = [Reg::RBX, Reg::R12, Reg::R13, Reg::R14, Reg::R15];

/// Refer to [this](https://blog.rchapman.org/posts/Linux_System_Call_Table_for_x86_64/) table and
/// [this](https://man7.org/linux/man-pages/man2/intro.2.html) manual when writing Linux syscalls.
pub const SYSCALL_REGS: [Reg; 7] = [
    Reg::RAX,
    Reg::RDI,
    Reg::RSI,
    Reg::RDX,
    Reg::R10,
    Reg::R8,
    Reg::R9,
];

#[derive(Debug, Hash, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Display)]
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
