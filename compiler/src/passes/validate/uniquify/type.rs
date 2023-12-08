use crate::passes::parse::types::Type;
use crate::passes::parse::Spanned;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::uniquify::try_get;
use crate::utils::push_map::PushMap;
use crate::utils::unique_sym::UniqueSym;

pub fn uniquify_type<'p>(
    typ: Type<Spanned<&'p str>>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Type<Spanned<UniqueSym<'p>>>, TypeError> {
    let typ = match typ {
        Type::Int(int) => Type::Int(int),
        Type::Bool => Type::Bool,
        Type::Unit => Type::Unit,
        Type::Never => Type::Never,
        Type::Fn { params, typ } => Type::Fn {
            params: params
                .into_iter()
                .map(|param| uniquify_type(param, scope))
                .collect::<Result<_, _>>()?,
            typ: Box::new(uniquify_type(*typ, scope)?),
        },
        Type::Var { sym } => Type::Var {
            sym: try_get(sym, scope)?,
        },
    };

    Ok(typ)
}
