use petgraph::algo::{toposort};
use petgraph::Directed;
use petgraph::prelude::GraphMap;
use crate::passes::parse::{Def, TypeDef};
use crate::passes::parse::types::Type;
use crate::passes::validate::{PrgTypeChecked, ValidateError};

impl<'p> PrgTypeChecked<'p> {
    pub fn check_sized(&self) -> Result<(), ValidateError> {
        let mut size_graph: GraphMap<&str, (), Directed> = GraphMap::new();
        for (_, def) in &self.defs {
            match def {
                Def::TypeDef { sym, def } => {
                    match def {
                        TypeDef::Struct { fields } => {
                            for (_, field) in fields {
                                match field {
                                    Type::Int | Type::Bool | Type::Unit | Type::Never | Type::Fn { .. } => {}
                                    Type::Var { sym: field_sym } => {
                                        size_graph.add_edge(sym, field_sym, ());
                                    }
                                }
                            }
                        }
                        TypeDef::Enum { .. } => todo!(),
                    }
                }
                _ => {}
            }
        }

        match toposort(&size_graph, None) {
            Ok(_) => Ok(()),
            Err(cycle) => Err(ValidateError::UnsizedType { sym: cycle.node_id().to_string() }),
        }
    }
}
