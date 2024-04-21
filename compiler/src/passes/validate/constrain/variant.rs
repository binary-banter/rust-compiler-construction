use crate::passes::parse::{Constrained, Span, Spanned, TypeDef};
use crate::passes::validate::constrain::expr::constrain_expr;
use crate::passes::validate::constrain::uncover_globals::{Env, EnvEntry};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, ExprUniquified, MetaConstrained};
use crate::utils::unique_sym::UniqueSym;

pub fn constraint_variant<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    enum_sym: Spanned<UniqueSym<'p>>,
    variant_sym: Spanned<&'p str>,
    bdy: Spanned<ExprUniquified<'p>>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    // Get the `EnvEntry` from the scope.
    // This should exist after uniquify, but could potentially not be an enum definition.
    let EnvEntry::Def {
        def: TypeDef::Enum {
            variants: def_variants,
        },
    } = &env.scope[&enum_sym.inner]
    else {
        return Err(TypeError::SymbolShouldBeEnum { span: enum_sym.meta });
    };

    // Check if variant_sym exists
    let Some((def_span, variant_typ)) = def_variants
        .iter()
        .find(|(def_variant, _)| def_variant.inner == variant_sym.inner)
    else {
        return Err(TypeError::UnknownEnumVariant {
            sym: variant_sym.inner.to_string(),
            span: variant_sym.meta,
        });
    };
    let def_span = def_span.meta;
    let variant_typ = variant_typ.clone();

    // Check body type
    let bdy_typ = constrain_expr(bdy, env)?;
    env.uf.expect_type(
        bdy_typ.meta.index,
        variant_typ,
        |field_typ, def_typ| TypeError::MismatchedStructField {
            expect: def_typ,
            got: field_typ,
            span_expected: def_span,
            span_got: bdy_typ.meta.span,
        },
    )?;

    let index = env.uf.add(PartialType::Var {
        sym: enum_sym.inner,
    });

    Ok(Constrained {
        meta: MetaConstrained { span, index },
        inner: ExprConstrained::Variant {
            enum_sym,
            variant_sym,
            bdy: Box::new(bdy_typ),
        },
    })
}
