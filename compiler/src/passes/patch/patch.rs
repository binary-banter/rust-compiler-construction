use crate::passes::assign::{Arg, FunAssigned, X86Assigned};
use crate::passes::patch::X86Patched;
use crate::passes::select::{Block, Instr};
use crate::utils::unique_sym::UniqueSym;
use crate::{add, mov, pop, push, reg, sub, time};
use functor_derive::Functor;

impl<'p> X86Assigned<'p> {
    #[must_use]
    pub fn patch(self) -> X86Patched<'p> {
        let program = X86Patched {
            fns: self.fns.fmap(patch_fn),
            entry: self.entry,
        };

        // display!(&program, Patch); // todo
        time!("patch");

        program
    }
}

fn patch_fn(fun: FunAssigned) -> FunAssigned {
    FunAssigned {
        blocks: fun
            .blocks
            .into_iter()
            .map(|(lbl, block)| (lbl, patch_block(block)))
            .collect(),
        entry: fun.entry,
        exit: fun.exit,
        stack_space: fun.stack_space,
    }
}

fn patch_block(block: Block<'_, Arg>) -> Block<'_, Arg> {
    Block {
        instrs: block
            .instrs
            .into_iter()
            .flat_map(patch_instr)
            .collect::<Vec<_>>(),
    }
}

fn patch_instr(instr: Instr<Arg, UniqueSym<'_>>) -> Vec<Instr<Arg, UniqueSym<'_>>> {
    match instr {
        Instr::Add { src, dst } => patch_args(src, dst, |src, dst| add!(src, dst)),
        Instr::Sub { src, dst } => patch_args(src, dst, |src, dst| sub!(src, dst)),
        Instr::Mov { src, dst } => patch_args(src, dst, |src, dst| mov!(src, dst)),
        _ => vec![instr],
    }
}

fn patch_args<'p>(
    src: Arg,
    dst: Arg,
    op: fn(Arg, Arg) -> Instr<Arg, UniqueSym<'p>>,
) -> Vec<Instr<Arg, UniqueSym<'p>>> {
    match (&src, &dst) {
        (Arg::Deref { .. }, Arg::Deref { .. }) => vec![
            push!(reg!(R8)),
            mov!(src, reg!(R8)),
            op(reg!(R8), dst),
            pop!(reg!(R8)),
        ],
        _ => vec![op(src, dst)],
    }
}
