use crate::language::x86var::{AX86Program, Arg, Block, Instr, PX86Program, Reg};
use crate::{addq, movq, reg, subq};

impl<'p> AX86Program<'p> {
    pub fn patch(self) -> PX86Program<'p> {
        PX86Program {
            blocks: self
                .blocks
                .into_iter()
                .map(|(lbl, block)| (lbl, patch_block(block)))
                .collect(),
            stack_space: self.stack_space,
        }
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

fn patch_instr(instr: Instr<'_, Arg>) -> Vec<Instr<'_, Arg>> {
    match instr {
        Instr::Addq { src, dst } => patch_args(src, dst, |src, dst| addq!(src, dst)),
        Instr::Subq { src, dst } => patch_args(src, dst, |src, dst| subq!(src, dst)),
        Instr::Movq { src, dst } => patch_args(src, dst, |src, dst| movq!(src, dst)),
        _ => vec![instr],
    }
}

fn patch_args<'p>(src: Arg, dst: Arg, op: fn(Arg, Arg) -> Instr<'p, Arg>) -> Vec<Instr<'p, Arg>> {
    match (&src, &dst) {
        (Arg::Deref { .. }, Arg::Deref { .. }) => vec![movq!(src, reg!(RAX)), op(reg!(RAX), dst)],
        _ => vec![op(src, dst)],
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::language::x86var::X86VarProgram;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn patch_instructions([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program: X86VarProgram = program
            .uniquify()
            .remove_complex_operands()
            .explicate()
            .select()
            .assign_homes()
            .patch()
            .into();
        let mut io = TestIO::new(input);
        let result = program.interpret("core", &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as patch_instructions => patch_instructions }
}
