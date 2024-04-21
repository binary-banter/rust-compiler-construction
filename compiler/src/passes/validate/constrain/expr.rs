use crate::passes::parse::{Constrained, Expr, Spanned};
use crate::passes::validate::constrain::access_field::constrain_access_field;
use crate::passes::validate::constrain::apply::constrain_apply;

use crate::passes::validate::constrain::asm::constrain_asm;
use crate::passes::validate::constrain::assign::constrain_assign;
use crate::passes::validate::constrain::binary_op::constrain_binary_op;
use crate::passes::validate::constrain::lit::constrain_lit;
use crate::passes::validate::constrain::r#break::constrain_break;
use crate::passes::validate::constrain::r#continue::constrain_continue;
use crate::passes::validate::constrain::r#if::constrain_if;
use crate::passes::validate::constrain::r#let::constrain_let;
use crate::passes::validate::constrain::r#loop::constrain_loop;
use crate::passes::validate::constrain::r#return::constrain_return;
use crate::passes::validate::constrain::r#struct::constrain_struct;
use crate::passes::validate::constrain::seq::constrain_seq;
use crate::passes::validate::constrain::unary_op::constrain_unary_op;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::constrain::var::constrain_var;
use crate::passes::validate::constrain::variant;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::{ExprConstrained, ExprUniquified};

pub fn constrain_expr<'p>(
    expr: Spanned<ExprUniquified<'p>>,
    env: &mut Env<'_, 'p>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    let span = expr.meta;

    match expr.inner {
        Expr::Lit { val } => constrain_lit(env, span, val),
        Expr::Var { sym } => constrain_var(env, span, sym),
        Expr::UnaryOp { op, expr } => constrain_unary_op(env, span, op, *expr),
        Expr::BinaryOp {
            op,
            exprs: [lhs, rhs],
        } => constrain_binary_op(env, span, op, *lhs, *rhs),
        Expr::Let {
            sym,
            mutable,
            typ,
            bnd,
            bdy,
        } => constrain_let(env, span, sym, mutable, typ, *bnd, *bdy),
        Expr::If { cnd, thn, els } => constrain_if(env, span, *cnd, *thn, *els),
        Expr::Apply { fun, args } => constrain_apply(env, span, *fun, args),
        Expr::Loop { bdy } => constrain_loop(env, span, *bdy),
        Expr::Break { bdy } => constrain_break(env, span, *bdy),
        Expr::Continue => constrain_continue(env, span),
        Expr::Return { bdy } => constrain_return(env, span, *bdy),
        Expr::Seq { stmt, cnt } => constrain_seq(env, span, *stmt, *cnt),
        Expr::Assign { sym, bnd } => constrain_assign(env, span, sym, *bnd),
        Expr::Struct { sym, fields } => constrain_struct(env, span, sym, fields),
        Expr::AccessField { strct, field } => constrain_access_field(env, span, *strct, field),
        Expr::Asm { instrs } => constrain_asm(env, span, instrs),
        Expr::Variant {
            enum_sym,
            variant_sym,
            bdy,
        } => variant::constraint_variant(env, span, enum_sym, variant_sym, *bdy),
        Expr::Switch { .. } => todo!(),
    }
}
