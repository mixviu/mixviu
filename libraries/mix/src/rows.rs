// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded row expression plans. Policy is the caller's Mix expression.
use crate::{
    ast::{BinOp, Expr, FunctionBody},
    error::{MixError, MixResult},
    evaluator::Evaluator,
    value::Value,
};
use indexmap::IndexMap;
use std::{future::Future, pin::Pin, sync::atomic::Ordering};
fn fail(s: impl Into<String>) -> MixError {
    MixError::structured("ROW_EXPRESSION", s.into())
}
enum Plan {
    Literal(Value),
    Row,
    Position,
    Field(Box<Plan>, String, bool),
    Index(Box<Plan>, Box<Plan>),
    Map(Vec<(String, Plan)>),
    List(Vec<Plan>),
    Binary(Box<Plan>, BinOp, Box<Plan>),
    Choose(Box<Plan>, Box<Plan>, Box<Plan>),
    Call(String, Vec<Plan>),
}
fn compile(
    e: &Expr,
    param: &str,
    index_param: Option<&str>,
    captures: Option<&std::collections::HashMap<String, Value>>,
    depth: usize,
    nodes: &mut usize,
    eval: &Evaluator,
) -> MixResult<Plan> {
    *nodes += 1;
    if depth > 64 || *nodes > 512 {
        return Err(fail("row expression exceeds 64 levels / 512 nodes"));
    }
    let mut sub = |e: &Expr| compile(e, param, index_param, captures, depth + 1, nodes, eval);
    Ok(match e {
        Expr::Variable(n) if n==param => Plan::Row,
        Expr::Variable(n) if index_param==Some(n.as_str()) => Plan::Position,
        Expr::Variable(n) => Plan::Literal(captures.and_then(|m|m.get(n)).ok_or_else(||fail(format!("uncaptured variable ${n}; pass constants through the lambda's lexical capture")))?.clone()),
        Expr::StringLiteral(s)|Expr::EscapedQuoteStringLiteral(s)=>Plan::Literal(Value::String(s.clone())),
        Expr::NumberLiteral(n)=>Plan::Literal(Value::Number(*n)),Expr::BoolLiteral(b)=>Plan::Literal(Value::Bool(*b)),Expr::NilLiteral=>Plan::Literal(Value::Nil),
        Expr::FieldAccess{object,field,optional}=>Plan::Field(Box::new(sub(object)?),field.clone(),*optional),
        Expr::Index{object,index}=>Plan::Index(Box::new(sub(object)?),Box::new(sub(index)?)),
        Expr::MapLiteral(fields)=>Plan::Map(fields.iter().map(|(k,v)|Ok((k.clone(),sub(v)?))).collect::<MixResult<_>>()?),
        Expr::ListLiteral(xs)=>Plan::List(xs.iter().map(&mut sub).collect::<MixResult<_>>()?),
        Expr::BinaryOp{left,op,right} if !matches!(op,BinOp::Pipe)=>Plan::Binary(Box::new(sub(left)?),op.clone(),Box::new(sub(right)?)),
        Expr::Ternary{cond,then_branch,else_branch}=>Plan::Choose(Box::new(sub(cond)?),Box::new(sub(then_branch)?),Box::new(sub(else_branch)?)),
        Expr::FunctionCall{name,args} if ["fmt","date_format","lower","upper","basename","dirname","split","len","contains","starts_with","ends_with","re_replace","slice","to_string","floor","abs","min","max"].contains(&name.as_str()) => {
            eval.check_capability(name)?;
            Plan::Call(name.clone(),args.iter().map(&mut sub).collect::<MixResult<_>>()?)
        }
        _=>return Err(fail("rows_project requires an expression lambda: literals, row fields/indexes, maps/lists, operators, ternary and documented pure calls; use map for other Mix code")),
    })
}
impl Plan {
    fn run(&self, row: &Value, position: usize, eval: &Evaluator) -> MixResult<Value> {
        let value = match self {
            Self::Literal(v) => v.clone(),
            Self::Row => row.clone(),
            Self::Position => Value::Number(position as f64),
            Self::Field(p, key, optional) => match p.run(row, position, eval)? {
                Value::Map(ref m) => m.get(key).cloned().unwrap_or(Value::Nil),
                Value::Nil if *optional => Value::Nil,
                v => {
                    return Err(fail(format!(
                        "cannot access field {key} on {}",
                        v.type_name()
                    )));
                }
            },
            Self::Index(p, i) => {
                Evaluator::index_value_clone(&p.run(row, position, eval)?, &i.run(row, position, eval)?)?
            }
            Self::List(xs) => Value::list(
                xs.iter()
                    .map(|p| p.run(row, position, eval))
                    .collect::<MixResult<_>>()?,
            ),
            Self::Map(xs) => Value::map(
                xs.iter()
                    .map(|(k, p)| Ok((k.clone(), p.run(row, position, eval)?)))
                    .collect::<MixResult<IndexMap<_, _>>>()?,
            ),
            Self::Choose(c, a, b) => {
                if c.run(row, position, eval)?.is_truthy() {
                    a.run(row, position, eval)?
                } else {
                    b.run(row, position, eval)?
                }
            }
            Self::Binary(a, op, b) => {
                let mut a = a.run(row, position, eval)?;
                match op {
                    BinOp::NilCoalesce if !matches!(a, Value::Nil) => a,
                    BinOp::And if !a.is_truthy() => a,
                    BinOp::Or if a.is_truthy() => a,
                    BinOp::NilCoalesce | BinOp::And | BinOp::Or => b.run(row, position, eval)?,
                    _ => eval.eval_binop(op, &mut a, &b.run(row, position, eval)?)?,
                }
            }
            Self::Call(name, xs) => eval.bulk_call(
                name,
                xs.iter()
                    .map(|p| p.run(row, position, eval))
                    .collect::<MixResult<_>>()?,
            )?,
        };
        eval.check_collection_size(&value)?;
        Ok(value)
    }
}
pub(crate) fn project<'a>(
    eval: &'a mut Evaluator,
    args: Vec<Value>,
) -> Pin<Box<dyn Future<Output = MixResult<Value>> + 'a>> {
    Box::pin(async move {
        let [Value::List(rows), Value::Function(f)] = args.as_slice() else {
            return Err(fail("rows_project(rows, fn(row) = expression)"));
        };
        if rows.len() > 65536 || !(1..=2).contains(&f.params.len()) {
            return Err(fail("at most 65536 rows; one row parameter and optional zero-based index parameter"));
        }
        let FunctionBody::Expression(expr) = &f.body else {
            return Err(fail("expression lambda required"));
        };
        let plan = compile(
            expr,
            &f.params[0].name,
            f.params.get(1).map(|p| p.name.as_str()),
            f.captures.as_ref(),
            0,
            &mut 0,
            eval,
        )?;
        let flag = eval.interrupt_flag();
        let mut out = Vec::with_capacity(rows.len());
        for (position, row) in rows.iter().enumerate() {
            if flag.load(Ordering::Acquire) {
                return Err(fail("row operation interrupted"));
            }
            eval.bulk_checkpoint()?;
            let value = plan.run(row, position, eval)?;
            eval.check_collection_size(&value)?;
            out.push(value);
        }
        Ok(Value::list(out))
    })
}
