// Copyright 2019 Jeremy Wall
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::ast::Position;
use crate::ast::{BinaryExprType, Expression, FormatArgs, Statement, Value};
use crate::build::format::{
    ExpressionFormatter, FormatRenderer, SimpleTemplate, TemplateParser, TemplatePart,
};
use crate::build::opcode::Primitive;
use crate::build::opcode::Value::{C, F, M, P, T};
use crate::build::opcode::{Hook, Op};

pub struct AST();

impl AST {
    pub fn translate(stmts: Vec<Statement>) -> Vec<Op> {
        let mut ops = Vec::new();
        for stmt in stmts {
            match stmt {
                Statement::Expression(expr) => Self::translate_expr(expr, &mut ops),
                Statement::Assert(_, _) => {
                    unimplemented!("Assert statements are not implmented yet")
                }
                Statement::Let(def) => {
                    let binding = def.name.fragment;
                    ops.push(Op::Sym(binding));
                    Self::translate_expr(def.value, &mut ops);
                    ops.push(Op::Bind);
                }
                Statement::Output(_, _, _) => {
                    unimplemented!("Out statements are not implmented yet")
                }
                Statement::Print(_, _, _) => {
                    unimplemented!("Print statements are not implmented yet")
                }
            }
        }
        return ops;
    }

    fn translate_expr(expr: Expression, mut ops: &mut Vec<Op>) {
        match expr {
            Expression::Simple(v) => {
                Self::translate_value(v, &mut ops);
            }
            Expression::Binary(def) => {
                match def.kind {
                    BinaryExprType::Add => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Add);
                    }
                    BinaryExprType::Sub => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Sub);
                    }
                    BinaryExprType::Div => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Div);
                    }
                    BinaryExprType::Mul => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Mul);
                    }
                    BinaryExprType::Equal => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Equal);
                    }
                    BinaryExprType::GT => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Gt);
                    }
                    BinaryExprType::LT => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Lt);
                    }
                    BinaryExprType::GTEqual => {
                        // An Equal and an And
                        //ops.push(Op::GtEqual);
                        unimplemented!("Binary expressions are not implmented yet")
                    }
                    BinaryExprType::LTEqual => {
                        //ops.push(Op::LtEqual);
                        unimplemented!("Binary expressions are not implmented yet")
                    }
                    BinaryExprType::NotEqual => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Equal);
                        ops.push(Op::Not);
                    }
                    BinaryExprType::REMatch => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Runtime(Hook::Regex));
                    }
                    BinaryExprType::NotREMatch => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Runtime(Hook::Regex));
                        ops.push(Op::Not);
                    }
                    BinaryExprType::IS => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Typ);
                        ops.push(Op::Equal);
                    }
                    BinaryExprType::AND => {
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Noop);
                        let idx = ops.len() - 1;
                        Self::translate_expr(*def.right, &mut ops);
                        let jptr = (ops.len() - 1 - idx) as i32;
                        ops[idx] = Op::And(dbg!(jptr));
                        dbg!(ops);
                    }
                    BinaryExprType::OR => {
                        // FIXME(jwall): This needs to be handled very differently
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Noop); // Placeholder used for
                        let idx = ops.len() - 1;
                        Self::translate_expr(*def.right, &mut ops);
                        let jptr = (ops.len() - 1 - idx) as i32;
                        ops[idx] = Op::Or(jptr);
                    }
                    BinaryExprType::Mod => {
                        Self::translate_expr(*def.right, &mut ops);
                        Self::translate_expr(*def.left, &mut ops);
                        ops.push(Op::Mod);
                    }
                    BinaryExprType::IN => {
                        unimplemented!("Binary expressions are not implmented yet")
                    }
                    BinaryExprType::DOT => {
                        // Dot expressions expect the left side to be pushed first
                        Self::translate_expr(*def.left, &mut ops);
                        // Symbols on the right side should be converted to strings to satisfy
                        // the Index operation contract.
                        match *def.right {
                            Expression::Simple(Value::Symbol(name)) => {
                                Self::translate_expr(
                                    Expression::Simple(Value::Str(name)),
                                    &mut ops,
                                );
                            }
                            expr => {
                                Self::translate_expr(expr, &mut ops);
                            }
                        }
                        ops.push(Op::Index);
                    }
                };
            }
            Expression::Grouped(expr, _) => {
                Self::translate_expr(*expr, &mut ops);
            }
            Expression::Fail(_) => unimplemented!("Fail expressions are not implmented yet"),
            Expression::Format(def) => {
                // TODO(jwall): It would actually be safer if this was happening
                // when we create the format def instead of here.
                match def.args {
                    FormatArgs::List(mut elems) => {
                        let formatter = SimpleTemplate::new();
                        let mut parts = dbg!(formatter.parse(&def.template));
                        // We need to push process these in reverse order for the
                        // vm to process things correctly;
                        elems.reverse();
                        parts.reverse();
                        let mut elems_iter = elems.drain(0..);
                        let mut parts_iter = parts.drain(0..);
                        Self::translate_template_part(
                            parts_iter.next().unwrap(),
                            &mut elems_iter,
                            &mut ops,
                            true,
                        );
                        for p in parts_iter {
                            Self::translate_template_part(p, &mut elems_iter, &mut ops, true);
                            ops.push(Op::Add);
                        }
                    }
                    FormatArgs::Single(e) => {
                        // TODO(jwall): Use expression formatter here.
                    }
                }
            }
            Expression::Func(_) => unimplemented!("Func expressions are not implmented yet"),
            Expression::FuncOp(_) => unimplemented!("FuncOp expressions are not implmented yet"),
            Expression::Import(_) => unimplemented!("Import expressions are not implmented yet"),
            Expression::Include(_) => unimplemented!("Include expressions are not implmented yet"),
            Expression::Module(_) => unimplemented!("Module expressions are not implmented yet"),
            Expression::Not(def) => {
                Self::translate_expr(*def.expr, &mut ops);
                ops.push(Op::Not);
            }
            Expression::Range(_) => unimplemented!("Range expressions are not implmented yet"),
            Expression::Select(_) => unimplemented!("Select expressions are not implmented yet"),
            Expression::Call(_) => unimplemented!("Call expressions are not implmented yet"),
            Expression::Copy(_) => unimplemented!("Copy expressions are not implmented yet"),
            Expression::Debug(_) => unimplemented!("Debug expressions are not implmented yet"),
        }
    }

    fn translate_template_part<EI: Iterator<Item = Expression>>(
        part: TemplatePart,
        elems: &mut EI,
        mut ops: &mut Vec<Op>,
        place_holder: bool,
    ) {
        match part {
            TemplatePart::Str(s) => {
                ops.push(Op::Val(Primitive::Str(s.into_iter().collect())));
            }
            TemplatePart::PlaceHolder(_idx) => {
                if !place_holder {
                    // In theory this should never be reachable
                    unreachable!();
                } else {
                    Self::translate_expr(dbg!(elems.next().unwrap()), &mut ops);
                    ops.push(Op::Render);
                }
            }
            TemplatePart::Expression(_expr) => {
                // TODO(jwall): We need to parse this.
                if place_holder {
                    unreachable!();
                } else {
                    unimplemented!("Expression Formatters are unimmplemented");
                }
            }
        }
    }

    fn translate_value(value: Value, mut ops: &mut Vec<Op>) {
        match value {
            Value::Int(i) => ops.push(Op::Val(Primitive::Int(i.val))),
            Value::Float(f) => ops.push(Op::Val(Primitive::Float(f.val))),
            Value::Str(s) => ops.push(Op::Val(Primitive::Str(s.val))),
            Value::Empty(_pos) => ops.push(Op::Val(Primitive::Empty)),
            Value::Boolean(b) => ops.push(Op::Val(Primitive::Bool(b.val))),
            Value::Symbol(s) => {
                ops.push(Op::DeRef(s.val));
            }
            Value::Tuple(flds) => {
                ops.push(Op::InitTuple);
                for (k, v) in flds.val {
                    ops.push(Op::Sym(k.fragment));
                    Self::translate_expr(v, &mut ops);
                    ops.push(Op::Field);
                }
            }
            Value::List(els) => {
                ops.push(Op::InitList);
                for el in els.elems {
                    Self::translate_expr(el, &mut ops);
                    ops.push(Op::Element);
                }
            }
        }
    }
}
