use crate::passes::atomize::Atom;
use crate::passes::eliminate::{ExprEliminated, FunEliminated, PrgEliminated, TailEliminated};
use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Lit, Meta, UnaryOp};
use crate::passes::select::{
    Block, Cnd, FunSelected, InstrSelected, VarArg, X86Selected, CALLEE_SAVED_NO_STACK,
    CALLER_SAVED,
};

use crate::utils::unique_sym::{gen_sym, UniqueSym};
use crate::*;
use std::collections::HashMap;

impl<'p> PrgEliminated<'p> {
    #[must_use]
    pub fn select(self) -> X86Selected<'p> {
        let fns = self
            .fns
            .into_iter()
            .map(|(sym, fun)| (sym, select_fun(fun)))
            .collect();

        let program = X86Selected {
            fns,
            entry: self.entry,
        };

        display!(&program, Select);
        time!("select");

        program
    }
}

fn select_fun(fun: FunEliminated) -> FunSelected {
    let mut blocks = HashMap::new();

    // Function entry & exit
    let entry = entry_block(&fun, &mut blocks);
    let exit = exit_block(&fun, &mut blocks);

    for (block_sym, block) in fun.blocks {
        let mut instrs = Vec::new();
        select_tail(block, &mut instrs, exit);
        blocks.insert(block_sym, Block { instrs });
    }

    FunSelected {
        blocks,
        entry,
        exit,
    }
}

/// Creates an entry block for the function.
fn entry_block<'p>(
    fun: &FunEliminated<'p>,
    blocks: &mut HashMap<UniqueSym<'p>, Block<'p, VarArg<UniqueSym<'p>>>>,
) -> UniqueSym<'p> {
    let entry = gen_sym("entry");
    let mut instrs = Vec::new();

    // Save stack pointers.
    instrs.push(push!(reg!(RBP)));
    instrs.push(mov!(reg!(RSP), reg!(RBP)));

    // Save callee-saved registers (excluding stack pointers).
    for reg in CALLEE_SAVED_NO_STACK {
        instrs.push(push!(VarArg::Reg(reg)));
    }

    // Prepare temporary stack space - this will be optimized in later passes.
    instrs.push(sub!(imm!(0x1000), reg!(RSP)));

    // Introduce parameters as local variables.
    for (reg, param) in CALLER_SAVED.into_iter().zip(fun.params.iter()) {
        // Sometimes not the entire register needs to be moved. But we considered this to be negligible.
        instrs.push(mov!(VarArg::Reg(reg), VarArg::XVar(param.sym)));
    }

    assert!(
        fun.params.len() <= 9,
        "Argument passing to stack is not yet implemented."
    );

    // Jump to core of the function - this was previously called "entry".
    instrs.push(jmp!(fun.entry));
    blocks.insert(entry, Block { instrs });
    entry
}

/// Creates an exit block for the function.
fn exit_block<'p>(
    fun: &FunEliminated<'p>,
    blocks: &mut HashMap<UniqueSym<'p>, Block<'p, VarArg<UniqueSym<'p>>>>,
) -> UniqueSym<'p> {
    let exit = gen_sym("exit");
    let mut instrs = Vec::new();

    // Restore temporary stack space.
    instrs.push(add!(imm!(0x1000), reg!(RSP)));

    // Restore callee-saved registers (excluding stack pointers).
    for reg in CALLEE_SAVED_NO_STACK.into_iter().rev() {
        instrs.push(pop!(VarArg::Reg(reg)));
    }

    // Restore stack pointers.
    instrs.push(pop!(reg!(RBP)));
    instrs.push(ret!());

    blocks.insert(exit, Block { instrs });
    exit
}

fn select_tail<'p>(
    tail: TailEliminated<'p>,
    instrs: &mut Vec<InstrSelected<'p>>,
    exit: UniqueSym<'p>,
) {
    match tail {
        TailEliminated::Return { exprs } => {
            assert!(
                exprs.len() <= 9,
                "Argument passing to stack is not yet implemented."
            );
            for (reg, arg) in CALLER_SAVED.into_iter().zip(exprs) {
                instrs.push(mov!(select_atom(arg), VarArg::Reg(reg)));
            }
            instrs.push(jmp!(exit));
        }
        TailEliminated::Seq {
            syms: sym,
            bnd,
            tail,
        } => {
            instrs.extend(select_assign(&sym, bnd));
            select_tail(*tail, instrs, exit);
        }
        TailEliminated::IfStmt { cnd, thn, els } => match cnd {
            ExprEliminated::BinaryOp {
                op,
                exprs: [expr_lhs, expr_rhs],
                ..
            } => {
                let tmp = gen_sym("tmp");
                instrs.extend(vec![
                    mov!(select_atom(expr_lhs), var!(tmp)),
                    cmp!(select_atom(expr_rhs), var!(tmp)),
                    jcc!(thn, select_cmp(op)),
                    jmp!(els),
                ]);
            }
            _ => unreachable!(),
        },
        TailEliminated::Goto { lbl } => {
            instrs.push(jmp!(lbl));
        }
    }
}

fn select_assign<'p>(
    dsts: &[UniqueSym<'p>],
    expr: Meta<Vec<Type<UniqueSym<'p>>>, ExprEliminated<'p>>,
) -> Vec<InstrSelected<'p>> {
    let dst = var!(dsts[0]);
    match expr.inner {
        ExprEliminated::Atom { atm, .. } => vec![mov!(select_atom(atm), dst)],
        ExprEliminated::Atom {
            atm: Atom::Var { sym },
            ..
        } => vec![mov!(var!(sym), dst)],
        ExprEliminated::BinaryOp {
            op,
            exprs: [a0, a1],
        } => match op {
            BinaryOp::Add => vec![
                mov!(select_atom(a0), dst.clone()),
                add!(select_atom(a1), dst),
            ],
            BinaryOp::Sub => vec![
                mov!(select_atom(a0), dst.clone()),
                sub!(select_atom(a1), dst),
            ],
            BinaryOp::Mul => vec![
                mov!(select_atom(a1), reg!(RAX)),
                mov!(select_atom(a0), reg!(RBX)),
                mul!(reg!(RBX)),
                mov!(reg!(RAX), dst),
            ],
            BinaryOp::Div => vec![
                mov!(imm!(0), reg!(RDX)),
                mov!(select_atom(a0), reg!(RAX)),
                mov!(select_atom(a1), reg!(RBX)),
                div!(reg!(RBX)),
                mov!(reg!(RAX), dst),
            ],
            BinaryOp::Mod => vec![
                mov!(imm!(0), reg!(RDX)),
                mov!(select_atom(a0), reg!(RAX)),
                mov!(select_atom(a1), reg!(RBX)),
                div!(reg!(RBX)),
                mov!(reg!(RDX), dst),
            ],
            BinaryOp::LAnd => vec![
                mov!(select_atom(a0), dst.clone()),
                and!(select_atom(a1), dst),
            ],
            BinaryOp::LOr => vec![
                mov!(select_atom(a0), dst.clone()),
                or!(select_atom(a1), dst),
            ],
            BinaryOp::Xor => vec![
                mov!(select_atom(a0), dst.clone()),
                xor!(select_atom(a1), dst),
            ],
            op @ (BinaryOp::GT
            | BinaryOp::GE
            | BinaryOp::EQ
            | BinaryOp::LE
            | BinaryOp::LT
            | BinaryOp::NE) => {
                let tmp = gen_sym("tmp");
                vec![
                    mov!(select_atom(a0), var!(tmp)),
                    cmp!(select_atom(a1), var!(tmp)),
                    mov!(imm!(0), reg!(RAX)), // todo: can be smaller
                    setcc!(select_cmp(op)),
                    mov!(reg!(RAX), dst),
                ]
            }
        },
        ExprEliminated::UnaryOp { op, expr: a0 } => match op {
            UnaryOp::Neg => vec![mov!(select_atom(a0), dst.clone()), neg!(dst)],
            UnaryOp::Not => vec![mov!(select_atom(a0), dst.clone()), xor!(imm!(1), dst)], // todo: can be smaller
        },
        ExprEliminated::FunRef { sym, .. } => vec![load_lbl!(sym, dst)],
        ExprEliminated::Apply { fun, args, .. } => {
            let mut instrs = vec![];

            for (arg, reg) in args.iter().zip(CALLER_SAVED.into_iter()) {
                instrs.push(mov!(select_atom(*arg), VarArg::Reg(reg)));
            }
            assert!(
                args.len() <= 9,
                "Argument passing to stack is not yet implemented."
            );

            instrs.push(call_indirect!(select_atom(fun), args.len()));

            for (reg, dst) in CALLER_SAVED.into_iter().zip(dsts) {
                instrs.push(mov!(VarArg::Reg(reg), var!(*dst)));
            }

            instrs
        }
        ExprEliminated::Asm { instrs } => instrs,
    }
}

fn select_atom(expr: Atom<'_>) -> VarArg<UniqueSym<'_>> {
    match expr {
        Atom::Val { val } => {
            match val {
                Lit::Int(int) => imm!(int),
                Lit::Bool(bool) => imm!(bool as i32), // todo: can be smaller
                Lit::Unit => imm!(0),                 // todo: can be smaller
            }
        }
        Atom::Var { sym } => var!(sym),
    }
}

fn select_cmp(op: BinaryOp) -> Cnd {
    match op {
        BinaryOp::GT => Cnd::GT,
        BinaryOp::GE => Cnd::GE,
        BinaryOp::EQ => Cnd::EQ,
        BinaryOp::LE => Cnd::LE,
        BinaryOp::LT => Cnd::LT,
        BinaryOp::NE => Cnd::NE,
        _ => unreachable!(),
    }
}
