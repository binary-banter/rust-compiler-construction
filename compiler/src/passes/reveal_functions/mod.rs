pub mod reveal_functions;

use crate::passes::parse::{Def, Expr, Lit, Op};
use crate::passes::uniquify::PrgUniquified;
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PrgRevealed<'p> {
    pub defs: HashMap<UniqueSym<'p>, Def<UniqueSym<'p>, RExpr<'p>>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug, PartialEq)]
pub enum RExpr<'p> {
    Lit {
        val: Lit,
    },
    Var {
        sym: UniqueSym<'p>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
    Prim {
        op: Op,
        args: Vec<RExpr<'p>>,
    },
    Let {
        sym: UniqueSym<'p>,
        bnd: Box<RExpr<'p>>,
        bdy: Box<RExpr<'p>>,
    },
    If {
        cnd: Box<RExpr<'p>>,
        thn: Box<RExpr<'p>>,
        els: Box<RExpr<'p>>,
    },
    Apply {
        fun: Box<RExpr<'p>>,
        args: Vec<RExpr<'p>>,
    },
    Loop {
        bdy: Box<RExpr<'p>>,
    },
    Break {
        bdy: Option<Box<RExpr<'p>>>,
    },
}

impl<'p> From<PrgRevealed<'p>> for PrgUniquified<'p> {
    fn from(value: PrgRevealed<'p>) -> Self {
        PrgUniquified {
            defs: value
                .defs
                .into_iter()
                .map(|(sym, def)| (sym, def.into()))
                .collect(),
            entry: value.entry,
        }
    }
}

//TODO also functor time?
impl<'p> From<Def<UniqueSym<'p>, RExpr<'p>>> for Def<UniqueSym<'p>, Expr<UniqueSym<'p>>> {
    fn from(value: Def<UniqueSym<'p>, RExpr<'p>>) -> Self {
        match value {
            Def::Fn {
                sym,
                params,
                typ,
                bdy,
            } => Def::Fn {
                sym,
                params,
                typ,
                bdy: bdy.into(),
            },
        }
    }
}

impl<'p> From<RExpr<'p>> for Expr<UniqueSym<'p>> {
    fn from(value: RExpr<'p>) -> Self {
        match value {
            RExpr::Lit { val } => Expr::Lit { val },
            RExpr::Prim { op, args } => Expr::Prim {
                op,
                args: args.into_iter().map(Into::into).collect(),
            },
            RExpr::Let { sym, bnd, bdy } => Expr::Let {
                sym,
                mutable: true,
                bnd: Box::new((*bnd).into()),
                bdy: Box::new((*bdy).into()),
            },
            RExpr::If { cnd, thn, els } => Expr::If {
                cnd: Box::new((*cnd).into()),
                thn: Box::new((*thn).into()),
                els: Box::new((*els).into()),
            },
            RExpr::Apply { fun, args } => Expr::Apply {
                fun: Box::new((*fun).into()),
                args: args.into_iter().map(Into::into).collect(),
            },
            RExpr::Var { sym } | RExpr::FunRef { sym } => Expr::Var { sym },
            RExpr::Loop { bdy } => Expr::Loop {
                bdy: Box::new((*bdy).into()),
            },
            RExpr::Break { bdy } => Expr::Break {
                bdy: bdy.map(|bdy| Box::new((*bdy).into())),
            },
        }
    }
}
