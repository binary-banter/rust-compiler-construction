use crate::utils::unique_sym::UniqueSym;

use crate::passes::assign::{LArg, LBlock, LFun, LX86VarProgram};
use crate::passes::select::{
    FunSelected, Instr, InstrSelected, Reg, VarArg, X86Selected, CALLER_SAVED, SYSCALL_REGS,
};
use functor_derive::Functor;
use petgraph::graphmap::DiGraphMap;
use petgraph::Direction;
use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap, HashSet};

impl<'p> X86Selected<'p> {
    #[must_use]
    pub(super) fn include_liveness(self) -> LX86VarProgram<'p> {
        LX86VarProgram {
            fns: self.fns.fmap(fn_liveness),
            entry: self.entry,
        }
    }
}

fn fn_liveness(fun: FunSelected) -> LFun {
    let graph = DiGraphMap::from_edges(fun.blocks.iter().flat_map(|(block_lbl, block)| {
        block.instrs.iter().filter_map(|instr| match instr {
            Instr::Jmp { lbl } | Instr::Jcc { lbl, .. } => Some((*block_lbl, *lbl, ())),
            _ => None,
        })
    }));

    let mut queue = BTreeSet::from([fun.exit]);
    let mut liveness = HashMap::from_iter(fun.blocks.fmap(|block| LBlock {
        live_after: vec![HashSet::new(); block.instrs.len()],
        instrs: block.instrs,
    }));
    let mut before_map = HashMap::new();

    while let Some(block_lbl) = queue.pop_first() {
        let prev_liveness = liveness.get_mut(&block_lbl).unwrap();

        let before = block_liveness(prev_liveness, &before_map);
        match before_map.entry(block_lbl) {
            Entry::Occupied(mut e) => {
                if e.get() != &before {
                    queue.extend(graph.neighbors_directed(block_lbl, Direction::Incoming));
                    e.insert(before);
                }
            }
            Entry::Vacant(e) => {
                queue.extend(graph.neighbors_directed(block_lbl, Direction::Incoming));
                e.insert(before);
            }
        }
    }

    LFun {
        blocks: liveness,
        entry: fun.entry,
        exit: fun.exit,
    }
}

fn block_liveness<'p>(
    block: &mut LBlock<'p>,
    before_map: &HashMap<UniqueSym<'p>, HashSet<LArg<'p>>>,
) -> HashSet<LArg<'p>> {
    let mut live = HashSet::new();

    for (i, instr) in block.instrs.iter().enumerate().rev() {
        let last_live = live.clone();

        handle_instr(instr, before_map, |arg, op| match (arg, op) {
            (VarArg::Imm { .. }, _) => {}
            (VarArg::Reg(reg), ReadWriteOp::Read | ReadWriteOp::ReadWrite) => {
                live.insert(LArg::Reg { reg: *reg });
            }
            (VarArg::Reg(reg), ReadWriteOp::Write) => {
                live.remove(&LArg::Reg { reg: *reg });
            }
            (VarArg::XVar(sym), ReadWriteOp::Read | ReadWriteOp::ReadWrite) => {
                live.insert(LArg::Var { sym: *sym });
            }
            (VarArg::XVar(sym), ReadWriteOp::Write) => {
                live.remove(&LArg::Var { sym: *sym });
            }
            (VarArg::Deref { reg, .. }, _) => {
                live.insert(LArg::Reg { reg: *reg });
            }
        });

        block.live_after[i] = last_live;
    }

    live
}

pub enum ReadWriteOp {
    Read,
    Write,
    ReadWrite,
}

pub fn handle_instr<'p>(
    instr: &InstrSelected<'p>,
    before_map: &HashMap<UniqueSym<'p>, HashSet<LArg<'p>>>,
    mut arg: impl FnMut(&VarArg<UniqueSym<'p>>, ReadWriteOp),
) {
    use ReadWriteOp::Read as R;
    use ReadWriteOp::ReadWrite as RW;
    use ReadWriteOp::Write as W;

    match instr {
        Instr::Add { src, dst, .. }
        | Instr::Sub { src, dst, .. }
        | Instr::And { src, dst, .. }
        | Instr::Or { src, dst, .. }
        | Instr::Xor { src, dst, .. } => {
            arg(dst, RW);
            arg(src, R);
        }
        Instr::Cmp { src, dst, .. } => {
            arg(dst, R);
            arg(src, R);
        }
        Instr::Mov { src, dst, .. } => {
            arg(dst, W);
            arg(src, R);
        }
        Instr::Push { src, .. } => {
            arg(src, R);
        }
        Instr::Pop { dst, .. } => {
            arg(dst, W);
        }
        Instr::Neg { dst, .. } | Instr::Not { dst, .. } => {
            arg(dst, RW);
        }
        Instr::CallDirect { arity, .. } => {
            for reg in CALLER_SAVED.into_iter().skip(*arity) {
                arg(&VarArg::Reg(reg), W);
            }
            for reg in CALLER_SAVED.into_iter().take(*arity) {
                arg(&VarArg::Reg(reg), RW);
            }
        }
        Instr::Syscall { arity } => {
            for reg in CALLER_SAVED {
                arg(&VarArg::Reg(reg), W);
            }
            for reg in SYSCALL_REGS.into_iter().take(*arity) {
                arg(&VarArg::Reg(reg), R);
            }
        }
        Instr::Ret { .. } => {
            // Because the return value of our function is in RAX, we need to consider it being read at the end of a block.
            arg(&VarArg::Reg(Reg::RAX), R);
        }
        Instr::Setcc { .. } => {
            arg(&VarArg::Reg(Reg::RAX), W);
        }
        Instr::Mul { src, .. } => {
            arg(&VarArg::Reg(Reg::RDX), W);
            arg(&VarArg::Reg(Reg::RAX), RW);
            arg(src, R);
        }
        Instr::Div { divisor, .. } => {
            arg(&VarArg::Reg(Reg::RDX), RW);
            arg(&VarArg::Reg(Reg::RAX), RW);
            arg(divisor, R);
        }
        Instr::Jmp { lbl } | Instr::Jcc { lbl, .. } => {
            for larg in before_map.get(lbl).unwrap_or(&HashSet::new()) {
                arg(&(*larg).into(), R);
            }
        }
        Instr::LoadLbl { dst, .. } => {
            arg(dst, W);
        }
        Instr::CallIndirect { src, arity } => {
            for reg in CALLER_SAVED.into_iter().skip(*arity) {
                arg(&VarArg::Reg(reg), W);
            }
            for reg in CALLER_SAVED.into_iter().take(*arity) {
                arg(&VarArg::Reg(reg), RW);
            }
            arg(src, R);
        }
        InstrSelected::IDiv { .. } => todo!(),
        InstrSelected::IMul { .. } => todo!(),
        InstrSelected::MovSX { .. } => todo!(),
    }
}
