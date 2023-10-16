//! This pass compiles `ULVarProgram`s  into `ALVarProgram` in which the arguments of operations are atomic expressions.
//!
//! This is accomplished by introducing new temporary variables, assigning
//! the complex operand to those new variables, and then using them in place
//! of the complex operand.
//!
//! We consider `Int`s and `Var`s atomic.

use crate::language::alvar::{AExpr, ALVarProgram, Atom};
use crate::language::lvar::{Expr, ULVarProgram};
use crate::passes::uniquify::{gen_sym, UniqueSym};

impl<'p> ULVarProgram<'p> {
    /// See module-level documentation.
    pub fn remove_complex_operands(self) -> ALVarProgram<'p> {
        ALVarProgram {
            defs: todo!(),
            bdy: rco_expr(self.bdy),
        }
    }
}

fn rco_expr(expr: Expr<UniqueSym<'_>>) -> AExpr<'_> {
    match expr {
        Expr::Val { val } => AExpr::Atom(Atom::Val { val }),
        Expr::Var { sym } => AExpr::Atom(Atom::Var { sym }),
        Expr::Prim { op, args } => {
            let (args, extras): (Vec<_>, Vec<_>) = args.into_iter().map(rco_atom).unzip();

            extras
                .into_iter()
                .flatten()
                .rfold(AExpr::Prim { op, args }, |bdy, (sym, bnd)| AExpr::Let {
                    sym,
                    bnd: Box::new(bnd),
                    bdy: Box::new(bdy),
                })
        }
        Expr::Let { sym, bnd, bdy } => AExpr::Let {
            sym,
            bnd: Box::new(rco_expr(*bnd)),
            bdy: Box::new(rco_expr(*bdy)),
        },
        Expr::If { cnd, thn, els } => AExpr::If {
            cnd: Box::new(rco_expr(*cnd)),
            thn: Box::new(rco_expr(*thn)),
            els: Box::new(rco_expr(*els)),
        },
    }
}

fn rco_atom(expr: Expr<UniqueSym<'_>>) -> (Atom<'_>, Option<(UniqueSym<'_>, AExpr<'_>)>) {
    match expr {
        Expr::Val { val } => (Atom::Val { val }, None),
        Expr::Var { sym } => (Atom::Var { sym }, None),
        Expr::Prim { .. } | Expr::Let { .. } | Expr::If { .. } => {
            let tmp = gen_sym("");
            (Atom::Var { sym: tmp }, Some((tmp, rco_expr(expr))))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::language::lvar::ULVarProgram;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn atomic([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program: ULVarProgram = program.uniquify().remove_complex_operands().into();
        let mut io = TestIO::new(input);
        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as remove_complex_operands => atomic }
}
