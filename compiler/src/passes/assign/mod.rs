mod assign;
mod color_interference;
mod compute_interference;
mod display;
mod include_liveness;

use crate::passes::select::{Block, FunSelected, Instr, InstrSelected, Reg, VarArg, X86Selected};
use crate::utils::unique_sym::UniqueSym;
use derive_more::Display;
use functor_derive::Functor;
use petgraph::graphmap::GraphMap;
use petgraph::Undirected;
use std::collections::{HashMap, HashSet};

pub struct X86Assigned<'p> {
    pub fns: HashMap<UniqueSym<'p>, FunAssigned<'p>>,
    pub entry: UniqueSym<'p>,
}

pub struct FunAssigned<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, Arg>>,
    pub entry: UniqueSym<'p>,
    pub exit: UniqueSym<'p>,
    pub stack_space: usize,
}

pub type InstrAssigned<'p> = Instr<Arg, UniqueSym<'p>>;

#[derive(Clone, Display)]
pub enum Arg {
    #[display(fmt = "${_0}")]
    Imm(i32),
    #[display(fmt = "%{_0}")]
    Reg(Reg),
    #[display(fmt = "[%{reg} + ${off}]")]
    Deref { reg: Reg, off: i64 },
}

pub struct InterferenceGraph<'p>(GraphMap<LArg<'p>, (), Undirected>);

pub struct LX86VarProgram<'p> {
    pub fns: HashMap<UniqueSym<'p>, LFun<'p>>,
    pub entry: UniqueSym<'p>,
}

pub struct LFun<'p> {
    pub blocks: HashMap<UniqueSym<'p>, LBlock<'p>>,
    pub entry: UniqueSym<'p>,
    pub exit: UniqueSym<'p>,
}

#[derive(PartialEq)]
pub struct LBlock<'p> {
    pub live_after: Vec<HashSet<LArg<'p>>>,
    pub instrs: Vec<InstrSelected<'p>>,
}

#[derive(Hash, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum LArg<'p> {
    Var { sym: UniqueSym<'p> },
    Reg { reg: Reg },
}

impl<'p> From<LArg<'p>> for VarArg<UniqueSym<'p>> {
    fn from(val: LArg<'p>) -> Self {
        match val {
            LArg::Var { sym } => VarArg::XVar(sym),
            LArg::Reg { reg } => VarArg::Reg(reg),
        }
    }
}

impl<'p> From<LBlock<'p>> for Block<'p, VarArg<UniqueSym<'p>>> {
    fn from(value: LBlock<'p>) -> Self {
        Block {
            instrs: value.instrs,
        }
    }
}

impl<'p> From<Arg> for VarArg<UniqueSym<'p>> {
    fn from(value: Arg) -> Self {
        match value {
            Arg::Imm(imm) => VarArg::Imm(imm),
            Arg::Reg(reg) => VarArg::Reg(reg),
            Arg::Deref { reg, off } => VarArg::Deref { reg, off },
        }
    }
}

impl<'p> From<FunAssigned<'p>> for FunSelected<'p> {
    fn from(value: FunAssigned<'p>) -> Self {
        FunSelected {
            blocks: value.blocks.fmap(|v| v.fmap(Into::into)),
            entry: value.entry,
            exit: value.exit,
        }
    }
}

impl<'p> From<X86Assigned<'p>> for X86Selected<'p> {
    fn from(value: X86Assigned<'p>) -> Self {
        X86Selected {
            fns: value.fns.fmap(Into::into),
            entry: value.entry,
        }
    }
}
