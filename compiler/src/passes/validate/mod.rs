pub mod validate;
pub mod error;
pub mod uniquify;
mod resolve_types;
mod solve_constraints;
mod generate_constraints;
mod check_sized;

use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Op};
use derive_more::Display;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct PrgValidated<'p> {
    pub defs: HashMap<&'p str, Def<'p, &'p str, TExpr<'p, &'p str>>>,
    pub entry: &'p str,
}

#[derive(Debug, PartialEq)]
pub enum TExpr<'p, A: Copy + Hash + Eq + Display> {
    Lit {
        val: TLit,
        typ: Type<A>,
    },
    Var {
        sym: A,
        typ: Type<A>,
    },
    Prim {
        op: Op,
        args: Vec<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Let {
        sym: A,
        bnd: Box<TExpr<'p, A>>,
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    If {
        cnd: Box<TExpr<'p, A>>,
        thn: Box<TExpr<'p, A>>,
        els: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Apply {
        fun: Box<TExpr<'p, A>>,
        args: Vec<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Loop {
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Break {
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Continue {
        typ: Type<A>,
    },
    Return {
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Seq {
        stmt: Box<TExpr<'p, A>>,
        cnt: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Assign {
        sym: A,
        bnd: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Struct {
        sym: A,
        fields: Vec<(&'p str, TExpr<'p, A>)>,
        typ: Type<A>,
    },
    Variant {
        enum_sym: A,
        variant_sym: A,
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    AccessField {
        strct: Box<TExpr<'p, A>>,
        field: &'p str,
        typ: Type<A>,
    },
    Switch {
        enm: Box<TExpr<'p, A>>,
        arms: Vec<(A, A, Box<TExpr<'p, A>>)>,
        typ: Type<A>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Display)]
pub enum TLit {
    #[display(fmt = "{val}")]
    Int { val: i32 },
    #[display(fmt = "{}", r#"if *val { "true" } else { "false" }"#)]
    Bool { val: bool },
    #[display(fmt = "unit")]
    Unit,
}

impl TLit {
    /// Returns the integer value if `TLit` is `Int`.
    /// # Panics
    /// Panics if `TLit` is not `Int`.
    #[must_use]
    pub fn int(self) -> i64 {
        if let TLit::Int { val } = self {
            val as i64
        } else {
            panic!()
        }
    }

    /// Returns the boolean value if `TLit` is `Bool`.
    /// # Panics
    /// Panics if `TLit` is not `Bool`.
    #[must_use]
    pub fn bool(self) -> bool {
        if let TLit::Bool { val } = self {
            val
        } else {
            panic!()
        }
    }
}

impl From<TLit> for i64 {
    fn from(value: TLit) -> Self {
        match value {
            TLit::Int { val } => val as i64,
            TLit::Bool { val } => val as i64,
            TLit::Unit => 0,
        }
    }
}

// This implementation is used by the parser.
impl FromStr for TLit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "false" => TLit::Bool { val: false },
            "true" => TLit::Bool { val: true },
            "unit" => TLit::Unit,
            s => TLit::Int {
                val: s.parse().map_err(|_| ())?,
            },
        })
    }
}

impl<'p, A: Copy + Hash + Eq + Display> TExpr<'p, A> {
    pub fn typ(&self) -> &Type<A> {
        match self {
            TExpr::Lit { typ, .. }
            | TExpr::Var { typ, .. }
            | TExpr::Prim { typ, .. }
            | TExpr::Let { typ, .. }
            | TExpr::If { typ, .. }
            | TExpr::Apply { typ, .. }
            | TExpr::Loop { typ, .. }
            | TExpr::Break { typ, .. }
            | TExpr::Continue { typ, .. }
            | TExpr::Return { typ, .. }
            | TExpr::Seq { typ, .. }
            | TExpr::Assign { typ, .. }
            | TExpr::Struct { typ, .. }
            | TExpr::Variant { typ, .. }
            | TExpr::AccessField { typ, .. }
            | TExpr::Switch { typ, .. } => typ,
        }
    }
}
