use std::collections::HashMap;
use once_cell::sync::Lazy;
use crate::passes::parse::{Meta, PrgParsed, Span};
use crate::passes::parse::types::Type;
use crate::passes::select::io::Std;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::DefUniquified;
use crate::passes::validate::error::TypeError::{NoMain, UndeclaredVar};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use crate::utils::push_map::PushMap;

mod typedef;
mod r#fn;
mod r#type;
mod expr;
mod def;

pub struct PrgUniquified<'p> {
    /// The global program definitions.
    pub defs: Vec<DefUniquified<'p>>,
    /// The symbol representing the entry point of the program.
    pub entry: UniqueSym<'p>,
    /// Entry points to functions from the standard library.
    pub std: Std<'p>,
}

pub static BUILT_INS: Lazy<HashMap<&'static str, Type<Meta<Span, UniqueSym<'static>>>>> =
    Lazy::new(|| {
        HashMap::from([
            (
                "exit",
                Type::Fn {
                    params: vec![Type::I64],
                    typ: Box::new(Type::Never),
                },
            ),
            (
                "print",
                Type::Fn {
                    params: vec![Type::I64],
                    typ: Box::new(Type::I64),
                },
            ),
            (
                "read",
                Type::Fn {
                    params: vec![],
                    typ: Box::new(Type::I64),
                },
            ),
        ])
    });

impl<'p> PrgParsed<'p> {
    pub fn uniquify(self) -> Result<PrgUniquified<'p>, TypeError> {
        let std: Std<'p> = BUILT_INS
            .iter()
            .map(|(sym, _)| (*sym, gen_sym(sym)))
            .collect();

        let mut scope = PushMap::from_iter(
            self.defs
                .iter()
                .map(|def| (def.sym().inner, gen_sym(def.sym().inner)))
                .chain(std.iter().map(|(&k, &v)| (k, v))),
        );

        let entry = *scope.get(&"main").ok_or(NoMain)?;

        Ok(PrgUniquified {
            defs: self
                .defs
                .into_iter()
                .map(|def| def::uniquify_def(def, &mut scope))
                .collect::<Result<_, _>>()?,
            entry,
            std,
        })
    }
}

fn try_get<'p>(
    sym: Meta<Span, &'p str>,
    scope: &PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Meta<Span, UniqueSym<'p>>, TypeError> {
    scope
        .get(&sym.inner)
        .ok_or(UndeclaredVar {
            sym: sym.inner.to_string(),
            span: sym.meta,
        })
        .map(|&inner| Meta {
            meta: sym.meta,
            inner,
        })
}

fn gen_spanned_sym(sym: Meta<Span, &str>) -> Meta<Span, UniqueSym> {
    Meta {
        inner: gen_sym(sym.inner),
        meta: sym.meta,
    }
}
