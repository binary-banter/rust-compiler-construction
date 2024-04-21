use crate::passes::parse::types::{IntType, Type};
use crate::passes::parse::{Constrained, Expr, Lit, Meta, Param, Span, Spanned, TypeDef, Typed};
use crate::passes::select::{Instr, InstrSelected, VarArg};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{
    DefConstrained, DefValidated, ExprConstrained, ExprValidated, InstrUniquified, PrgConstrained,
    PrgValidated,
};
use crate::utils::union_find::{UnionFind, UnionIndex};
use crate::utils::unique_sym::UniqueSym;
use crate::*;
use functor_derive::Functor;
use std::num::ParseIntError;

impl<'p> PrgConstrained<'p> {
    pub fn resolve(mut self) -> Result<PrgValidated<'p>, TypeError> {
        Ok(PrgValidated {
            defs: self
                .defs
                .into_iter()
                .map(|(sym, def)| resolve_def(def, &mut self.uf).map(|def| (sym, def)))
                .collect::<Result<_, _>>()?,
            entry: self.entry,
        })
    }
}

fn resolve_def<'p>(
    def: DefConstrained<'p>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Result<DefValidated<'p>, TypeError> {
    let def = match def {
        DefConstrained::Fn {
            sym,
            params,
            typ,
            bdy,
        } => DefValidated::Fn {
            sym: sym.inner,
            params: params.fmap(|p| Param {
                sym: p.sym.inner,
                typ: resolve_type(p.typ),
                mutable: p.mutable,
            }),
            typ: resolve_type(typ),
            bdy: resolve_expr(bdy, uf)?,
        },
        DefConstrained::TypeDef { sym, def } => DefValidated::TypeDef {
            sym: sym.inner,
            def: resolve_typedef(def),
        },
    };

    Ok(def)
}

fn resolve_typedef<'p>(
    typedef: TypeDef<Spanned<UniqueSym<'p>>, Spanned<&'p str>>,
) -> TypeDef<UniqueSym<'p>, &'p str> {
    match typedef {
        TypeDef::Struct { fields } => TypeDef::Struct {
            fields: fields
                .fmap(|(field_sym, field_typ)| (field_sym.inner, resolve_type(field_typ))),
        },
        TypeDef::Enum { .. } => todo!(),
    }
}

fn resolve_type(typ: Type<Spanned<UniqueSym>>) -> Type<UniqueSym> {
    match typ {
        Type::Int(int) => Type::Int(int),
        Type::Bool => Type::Bool,
        Type::Unit => Type::Unit,
        Type::Never => Type::Never,
        Type::Fn { params, typ } => Type::Fn {
            params: params.fmap(resolve_type),
            typ: typ.fmap(resolve_type),
        },
        Type::Var { sym } => Type::Var { sym: sym.inner },
    }
}

/// Panic if not possible
fn partial_type_to_type<'p>(
    value: UnionIndex,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Option<Type<UniqueSym<'p>>> {
    Some(match uf.get(value).clone() {
        PartialType::Int(int) => Type::Int(int),
        PartialType::IntAmbiguous => return None,
        PartialType::Bool => Type::Bool,
        PartialType::Unit => Type::Unit,
        PartialType::Never => Type::Never,
        PartialType::Var { sym } => Type::Var { sym },
        PartialType::Fn { params, typ } => Type::Fn {
            params: params
                .into_iter()
                .map(|param| partial_type_to_type(uf.find(param), uf))
                .collect::<Option<_>>()?,
            typ: Box::new(partial_type_to_type(typ, uf)?),
        },
    })
}

fn resolve_int_lit<T: From<u8>>(
    original_val: &str,
    span: Span,
    from_radix: fn(&str, u32) -> Result<T, ParseIntError>,
) -> Result<T, TypeError> {
    let mut val = original_val;
    if val.ends_with("i64") || val.ends_with("u64") {
        val = &val[..val.len() - 3];
    }

    let (base, val) = match val {
        s if s.starts_with('b') => {
            let mut s = s[1..].chars();

            let int = match (s.next(), s.next(), s.next(), s.next(), s.next()) {
                (Some('\''), Some(s), Some('\''), None, None) => T::from(s as u8),
                (Some('\''), Some('\\'), Some(s), Some('\''), None) => {
                    let s = match s {
                        'n' => '\n',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '\'' => '\'',
                        '0' => '\0',
                        s => {
                            return Err(TypeError::InvalidEscape {
                                span,
                                val: format!("\\{s}"),
                            })
                        }
                    };
                    T::from(s as u8)
                }
                _ => unreachable!("Congrats you made an invalid byte lit, plx tell us how"),
            };

            return Ok(int);
        }
        s if s.starts_with("0b") => (2, &s[2..]),
        s if s.starts_with("0o") => (8, &s[2..]),
        s if s.starts_with("0x") => (16, &s[2..]),
        s => (10, s),
    };

    from_radix(&val.replace('_', ""), base).map_err(|error| TypeError::InvalidInteger {
        span,
        val: original_val.to_string(),
        typ: "I64",
        err: error.to_string(),
    })
}

fn resolve_expr<'p>(
    expr: Constrained<ExprConstrained<'p>>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Result<Typed<'p, ExprValidated<'p>>, TypeError> {
    // Type of the expression, if `None` then type is still ambiguous.
    let typ = partial_type_to_type(expr.meta.index, uf);

    let expr = match expr.inner {
        Expr::Lit { val } => {
            let val = match val {
                Lit::Int(val) => match &typ {
                    Some(typ) => {
                        let int = match typ {
                            Type::Int(_int) => {
                                resolve_int_lit(val, expr.meta.span, i64::from_str_radix)?
                            }
                            //     match int {
                            //     IntType::I8 => todo!(),
                            //     IntType::U8 => Int::U8({
                            //         resolve_int_lit(val, expr.meta.span, u8::from_str_radix)?
                            //     }),
                            //     IntType::I16 => Int::I16(resolve_int_lit(
                            //         val,
                            //         expr.meta.span,
                            //         i16::from_str_radix,
                            //     )?),
                            //     IntType::U16 => Int::U16(resolve_int_lit(
                            //         val,
                            //         expr.meta.span,
                            //         u16::from_str_radix,
                            //     )?),
                            //     IntType::I32 => Int::I32(resolve_int_lit(
                            //         val,
                            //         expr.meta.span,
                            //         i32::from_str_radix,
                            //     )?),
                            //     IntType::U32 => Int::U32(resolve_int_lit(
                            //         val,
                            //         expr.meta.span,
                            //         u32::from_str_radix,
                            //     )?),
                            //     IntType::I64 => Int::I64(resolve_int_lit(
                            //         val,
                            //         expr.meta.span,
                            //         i64::from_str_radix,
                            //     )?),
                            //     IntType::U64 => Int::U64(resolve_int_lit(
                            //         val,
                            //         expr.meta.span,
                            //         u64::from_str_radix,
                            //     )?),
                            // },
                            _ => unreachable!(),
                        };
                        Lit::Int(int)
                    }
                    None => {
                        return Err(TypeError::IntegerAmbiguous {
                            span: expr.meta.span,
                        })
                    }
                },
                Lit::Bool(bool) => Lit::Bool(bool),
                Lit::Unit => Lit::Unit,
            };
            Expr::Lit { val }
        }
        Expr::Var { sym } => Expr::Var { sym: sym.inner },
        Expr::UnaryOp {
            op,
            expr: expr_inner,
        } => Expr::UnaryOp {
            op,
            expr: Box::new(resolve_expr(*expr_inner, uf)?),
        },
        Expr::BinaryOp {
            op,
            exprs: [e1, e2],
        } => Expr::BinaryOp {
            op,
            exprs: [resolve_expr(*e1, uf)?, resolve_expr(*e2, uf)?].map(Box::new),
        },
        Expr::Let {
            sym,
            mutable,
            typ,
            bnd,
            bdy,
        } => Expr::Let {
            sym: sym.inner,
            mutable,
            typ: typ.map(resolve_type),
            bnd: Box::new(resolve_expr(*bnd, uf)?),
            bdy: Box::new(resolve_expr(*bdy, uf)?),
        },
        Expr::If { cnd, thn, els } => Expr::If {
            cnd: Box::new(resolve_expr(*cnd, uf)?),
            thn: Box::new(resolve_expr(*thn, uf)?),
            els: Box::new(resolve_expr(*els, uf)?),
        },
        Expr::Apply { fun, args } => Expr::Apply {
            fun: Box::new(resolve_expr(*fun, uf)?),
            args: args
                .into_iter()
                .map(|arg| resolve_expr(arg, uf))
                .collect::<Result<_, _>>()?,
        },
        Expr::Loop { bdy } => Expr::Loop {
            bdy: Box::new(resolve_expr(*bdy, uf)?),
        },
        Expr::Break { bdy } => Expr::Break {
            bdy: Box::new(resolve_expr(*bdy, uf)?),
        },
        Expr::Continue => Expr::Continue,
        Expr::Return { bdy } => Expr::Return {
            bdy: Box::new(resolve_expr(*bdy, uf)?),
        },
        Expr::Seq { stmt, cnt } => Expr::Seq {
            stmt: Box::new(resolve_expr(*stmt, uf)?),
            cnt: Box::new(resolve_expr(*cnt, uf)?),
        },
        Expr::Assign { sym, bnd } => Expr::Assign {
            sym: sym.inner,
            bnd: Box::new(resolve_expr(*bnd, uf)?),
        },
        Expr::Struct { sym, fields } => Expr::Struct {
            sym: sym.inner,
            fields: fields
                .into_iter()
                .map(|(field_sym, field_bnd)| {
                    resolve_expr(field_bnd, uf).map(|bnd| (field_sym.inner, bnd))
                })
                .collect::<Result<_, _>>()?,
        },
        Expr::AccessField { strct, field } => Expr::AccessField {
            strct: Box::new(resolve_expr(*strct, uf)?),
            field: field.inner,
        },
        Expr::Variant { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
        ExprConstrained::Asm { instrs } => ExprValidated::Asm {
            instrs: instrs.into_iter().map(resolve_instr).collect(),
        },
    };

    Ok(Meta {
        meta: typ.unwrap(),
        inner: expr,
    })
}

pub fn resolve_instr<'p>(
    instr: Instr<VarArg<Spanned<UniqueSym<'p>>>, Spanned<UniqueSym<'p>>>,
) -> InstrSelected<'p> {
    let map = |arg: VarArg<Spanned<UniqueSym<'p>>>| match arg {
        VarArg::Imm(imm) => VarArg::Imm(imm),
        VarArg::Reg(reg) => VarArg::Reg(reg),
        VarArg::Deref { reg, off } => VarArg::Deref { reg, off },
        VarArg::XVar(sym) => VarArg::XVar(sym.inner),
    };

    match instr {
        InstrUniquified::Add { src, dst } => InstrSelected::Add {
            src: map(src),
            dst: map(dst),
        },
        InstrUniquified::Sub { src, dst } => InstrSelected::Sub {
            src: map(src),
            dst: map(dst),
        },
        InstrUniquified::Div { divisor } => InstrSelected::Div {
            divisor: map(divisor),
        },
        InstrUniquified::IDiv { divisor } => InstrSelected::IDiv {
            divisor: map(divisor),
        },
        InstrUniquified::Mul { src } => InstrSelected::Mul { src: map(src) },
        InstrUniquified::IMul { src } => InstrSelected::IMul { src: map(src) },
        InstrUniquified::Neg { dst } => InstrSelected::Neg { dst: map(dst) },
        InstrUniquified::Mov { src, dst } => InstrSelected::Mov {
            src: map(src),
            dst: map(dst),
        },
        InstrUniquified::MovSX { src, dst } => InstrSelected::MovSX {
            src: map(src),
            dst: map(dst),
        },
        InstrUniquified::Push { src } => InstrSelected::Push { src: map(src) },
        InstrUniquified::Pop { dst } => InstrSelected::Pop { dst: map(dst) },
        InstrUniquified::Syscall { arity } => InstrSelected::Syscall { arity },
        InstrUniquified::Cmp { src, dst } => InstrSelected::Cmp {
            src: map(src),
            dst: map(dst),
        },
        InstrUniquified::And { src, dst } => InstrSelected::And {
            src: map(src),
            dst: map(dst),
        },
        InstrUniquified::Or { src, dst } => InstrSelected::Or {
            src: map(src),
            dst: map(dst),
        },
        InstrUniquified::Xor { src, dst } => InstrSelected::Xor {
            src: map(src),
            dst: map(dst),
        },
        InstrUniquified::Not { dst } => InstrSelected::Not { dst: map(dst) },
        InstrUniquified::Setcc { .. }
        | InstrUniquified::Ret { .. }
        | InstrUniquified::Jmp { .. }
        | InstrUniquified::Jcc { .. }
        | InstrUniquified::LoadLbl { .. }
        | InstrUniquified::CallDirect { .. }
        | InstrUniquified::CallIndirect { .. } => unreachable!(),
    }
}
