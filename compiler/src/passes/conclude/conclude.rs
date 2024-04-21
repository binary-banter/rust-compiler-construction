use crate::passes::assign::{Arg, InstrAssigned};
use crate::passes::conclude::X86Concluded;
use crate::passes::patch::X86Patched;
use crate::passes::select::{Block, Instr};
use crate::utils::unique_sym::gen_sym;
use crate::*;
use std::collections::HashMap;


impl<'p> X86Patched<'p> {
    #[must_use]
    pub fn conclude(self) -> X86Concluded<'p> {
        let entries = self
            .fns
            .iter()
            .map(|(sym, f)| (*sym, f.entry))
            .collect::<HashMap<_, _>>();

        let mut blocks = self
            .fns
            .into_iter()
            .flat_map(|(_, mut fun)| {
                fix_stack_space(fun.blocks.get_mut(&fun.entry).unwrap(), fun.stack_space);
                fix_stack_space(fun.blocks.get_mut(&fun.exit).unwrap(), fun.stack_space);

                // Replace calls to function labels with calls to the entries of those functions.
                fun.blocks.into_iter().map(|(block_sym, mut block)| {
                    for instr in &mut block.instrs {
                        match instr {
                            Instr::CallDirect { lbl, .. } | Instr::LoadLbl { lbl, .. } => {
                                *lbl = entries[&lbl];
                            }
                            _ => {}
                        }
                    }
                    (block_sym, block)
                })
            })
            .collect::<HashMap<_, _>>();

        let entry = gen_sym("runtime");
        blocks.insert(
            entry,
            block!(
                call_direct!(entries[&self.entry], 0),
                mov!(reg!(RAX), reg!(RDI)),
                mov!(imm!(0x3C), reg!(RAX)), // todo: can be smaller
                syscall!(2)
            ),
        );

        let program = X86Concluded { blocks, entry };

        // display!(&program, Conclude); // todo
        time!("conclude");

        program
    }
}

/// Fixes stack allocation for spilled variables.
fn fix_stack_space(block: &mut Block<Arg>, stack_space: usize) {
    for instr in &mut block.instrs {
        match instr {
            InstrAssigned::Add {
                src: Arg::Imm(val),
                ..
            }
            | InstrAssigned::Sub {
                src: Arg::Imm(val),
                ..
            } => {
                assert_eq!(*val, 0x1000);
                *val = stack_space as i32;
            }
            InstrAssigned::Add {
                src: Arg::Imm(_), ..
            }
            | InstrAssigned::Sub {
                src: Arg::Imm(_), ..
            } => {
                todo!()
            }
            _ => {}
        }
    }
}
