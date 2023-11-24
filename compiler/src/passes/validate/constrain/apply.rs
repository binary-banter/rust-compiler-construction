use crate::passes::parse::{Meta, Span};
use crate::passes::validate::{CMeta, ExprConstrained, ExprUniquified};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::utils::expect::expect;

pub fn constrain_apply<'p>(env: &mut Env<'_, 'p>, span: Span, fun: Box<Meta<Span, ExprUniquified<'p>>>, args: Vec<Meta<Span, ExprUniquified<'p>>>) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    let fun = expr::constrain_expr(*fun, env)?;
    let args: Vec<_> = args
        .into_iter()
        .map(|arg| expr::constrain_expr(arg, env))
        .collect::<Result<_, _>>()?;

    let p_typ = env.uf.get(fun.meta.index).clone();
    let PartialType::Fn { params, typ } = p_typ else {
        return Err(TypeError::TypeMismatchExpectFn {
            got: p_typ.to_string(&mut env.uf),
            span_got: fun.meta.span,
        });
    };

    expect(
        params.len() == args.len(),
        TypeError::ArgCountMismatch {
            got: args.len(),
            expected: params.len(),
            span, // todo: maybe highlight only the args and params?
        },
    )?;

    for (arg, param_type) in args.iter().zip(params.iter()) {
        env.uf
            .expect_equal(arg.meta.index, *param_type, |arg_type, param_type| {
                TypeError::FnArgExpect {
                    arg: arg_type,
                    param: param_type,
                    span_arg: arg.meta.span,
                }
            })?;
    }

    Ok(Meta {
        meta: CMeta { span, index: typ },
        inner: ExprConstrained::Apply {
            fun: Box::new(fun),
            args,
        },
    })
}
