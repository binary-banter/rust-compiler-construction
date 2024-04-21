pub mod atomize;
mod display;

use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Def, Lit, Typed, UnaryOp};
use crate::passes::select::InstrSelected;

use crate::utils::unique_sym::UniqueSym;
use derive_more::Display;
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Display)]
#[display(fmt = "{}", r#"defs.values().format("\n")"#)]
pub struct PrgAtomized<'p> {
    pub defs: HashMap<UniqueSym<'p>, DefAtomized<'p>>,
    pub entry: UniqueSym<'p>,
}

pub type DefAtomized<'p> = Def<UniqueSym<'p>, &'p str, Typed<'p, AExpr<'p>>>;

pub enum AExpr<'p> {
    Atom {
        atm: Atom<'p>,
    },
    BinaryOp {
        op: BinaryOp,
        exprs: [Atom<'p>; 2],
    },
    UnaryOp {
        op: UnaryOp,
        expr: Atom<'p>,
    },
    Let {
        sym: UniqueSym<'p>,
        bnd: Box<Typed<'p, AExpr<'p>>>,
        bdy: Box<Typed<'p, AExpr<'p>>>,
    },
    If {
        cnd: Box<Typed<'p, AExpr<'p>>>,
        thn: Box<Typed<'p, AExpr<'p>>>,
        els: Box<Typed<'p, AExpr<'p>>>,
    },
    Apply {
        fun: Atom<'p>,
        args: Vec<(Atom<'p>, Type<UniqueSym<'p>>)>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
    Loop {
        bdy: Box<Typed<'p, AExpr<'p>>>,
    },
    Break {
        bdy: Box<Typed<'p, AExpr<'p>>>,
    },
    Continue,
    Seq {
        stmt: Box<Typed<'p, AExpr<'p>>>,
        cnt: Box<Typed<'p, AExpr<'p>>>,
    },
    Assign {
        sym: UniqueSym<'p>,
        bnd: Box<Typed<'p, AExpr<'p>>>,
    },
    Return {
        bdy: Box<Typed<'p, AExpr<'p>>>,
    },
    Struct {
        sym: UniqueSym<'p>,
        fields: Vec<(&'p str, Atom<'p>)>,
    },
    AccessField {
        strct: Atom<'p>,
        field: &'p str,
    },
    Asm {
        instrs: Vec<InstrSelected<'p>>,
    },
}

#[derive(Copy, Clone, Display)]
pub enum Atom<'p> {
    Val { val: Lit<i64> },
    Var { sym: UniqueSym<'p> },
}

impl<'p> Atom<'p> {
    pub fn var(self) -> UniqueSym<'p> {
        if let Atom::Var { sym } = self {
            sym
        } else {
            panic!()
        }
    }
}
