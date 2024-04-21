mod check_sized;
mod constrain;
pub mod error;
pub mod partial_type;
mod resolve;
#[cfg(test)]
mod tests;
mod uniquify;
pub mod validate;

use crate::passes::parse::types::Type;
use crate::passes::parse::{Constrained, Def, Expr, Lit, Span, Spanned, Typed};
use crate::passes::select::{Instr, VarArg};
use crate::utils::union_find::{UnionFind, UnionIndex};
use crate::utils::unique_sym::UniqueSym;
use derive_more::Display;
use itertools::Itertools;
use partial_type::PartialType;
use std::collections::HashMap;

#[derive(Debug, Display)]
#[display(fmt = "{}", r#"defs.values().format("\n")"#)]
pub struct PrgValidated<'p> {
    pub defs: HashMap<UniqueSym<'p>, DefValidated<'p>>,
    pub entry: UniqueSym<'p>,
}

pub struct PrgConstrained<'p> {
    pub defs: HashMap<UniqueSym<'p>, DefConstrained<'p>>,
    pub entry: UniqueSym<'p>,
    pub uf: UnionFind<PartialType<'p>>,
}

pub type DefValidated<'p> = Def<UniqueSym<'p>, &'p str, Typed<'p, ExprValidated<'p>>>;
pub type ExprValidated<'p> = Expr<UniqueSym<'p>, &'p str, Lit<i64>, Type<UniqueSym<'p>>>;

pub type DefConstrained<'p> =
    Def<Spanned<UniqueSym<'p>>, Spanned<&'p str>, Constrained<ExprConstrained<'p>>>;
pub type ExprConstrained<'p> =
    Expr<Spanned<UniqueSym<'p>>, Spanned<&'p str>, Lit<&'p str>, MetaConstrained>;

pub type DefUniquified<'p> =
    Def<Spanned<UniqueSym<'p>>, Spanned<&'p str>, Spanned<ExprUniquified<'p>>>;
pub type ExprUniquified<'p> = Expr<Spanned<UniqueSym<'p>>, Spanned<&'p str>, Lit<&'p str>, Span>;
pub type InstrUniquified<'p> = Instr<VarArg<Spanned<UniqueSym<'p>>>, Spanned<UniqueSym<'p>>>;

pub struct MetaConstrained {
    pub span: Span,
    pub index: UnionIndex,
}