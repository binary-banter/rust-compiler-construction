use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Spanned, TypeDef};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::PrgConstrained;
use crate::utils::unique_sym::UniqueSym;
use petgraph::algo::toposort;
use petgraph::prelude::GraphMap;
use petgraph::Directed;

impl<'p> PrgConstrained<'p> {
    pub fn check_sized(&self) -> Result<(), TypeError> {
        let mut size_graph: GraphMap<UniqueSym<'p>, (), Directed> = GraphMap::new();

        let mut add_to_size_graph =
            |sym: &Spanned<UniqueSym<'p>>, typ: &Type<Spanned<UniqueSym<'p>>>| match typ {
                Type::Int(_) | Type::Bool | Type::Unit | Type::Never | Type::Fn { .. } => {}
                Type::Var { sym: typ_sym } => {
                    size_graph.add_edge(sym.inner, typ_sym.inner, ());
                }
            };

        for def in self.defs.values() {
            #[allow(clippy::single_match)]
            match def {
                Def::TypeDef { sym, def } => match def {
                    TypeDef::Struct { fields } => {
                        for (_, field) in fields {
                            add_to_size_graph(sym, field)
                        }
                    }
                    TypeDef::Enum { variants } => {
                        for (_, variant) in variants {
                            add_to_size_graph(sym, variant)
                        }
                    }
                },
                _ => {}
            }
        }

        match toposort(&size_graph, None) {
            Ok(_) => Ok(()),
            Err(cycle) => Err(TypeError::UnsizedType {
                sym: cycle.node_id().to_string(),
                span: self.defs[&cycle.node_id()].sym().meta,
            }),
        }
    }
}
