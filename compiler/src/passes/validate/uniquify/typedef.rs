use crate::passes::parse::{Def, Spanned, TypeDef};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::uniquify::r#type::uniquify_type;
use crate::passes::validate::uniquify::try_get;
use crate::passes::validate::DefUniquified;
use crate::utils::push_map::PushMap;
use crate::utils::unique_sym::UniqueSym;

pub fn uniquify_typedef<'p>(
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
    sym: Spanned<&'p str>,
    def: TypeDef<Spanned<&'p str>, Spanned<&'p str>>,
) -> Result<DefUniquified<'p>, TypeError> {
    let def = match def {
        TypeDef::Struct { fields } => TypeDef::Struct {
            fields: fields
                .into_iter()
                .map(|(sym, typ)| Ok((sym, uniquify_type(typ, scope)?)))
                .collect::<Result<_, _>>()?,
        },
        TypeDef::Enum { variants } => TypeDef::Enum {
            variants: variants
                .into_iter()
                .map(|(sym, typ)| Ok((sym, uniquify_type(typ, scope)?)))
                .collect::<Result<_, _>>()?,
        },
    };

    Ok(Def::TypeDef {
        sym: try_get(sym, scope)?,
        def,
    })
}
