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
use crate::ast::{BinaryExprType, Expression, Statement, Value};
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
                Statement::Let(_) => unimplemented!("Let statements are not implmented yet"),
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
                    BinaryExprType::IN | BinaryExprType::DOT => {
                        unimplemented!("Binary expressions are not implmented yet")
                        // TODO
                    }
                };
            }
            Expression::Grouped(expr, _) => {
                Self::translate_expr(*expr, &mut ops);
            }
            Expression::Fail(_) => unimplemented!("Fail expressions are not implmented yet"),
            Expression::Format(_) => unimplemented!("Format expressions are not implmented yet"),
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

    fn translate_value(value: Value, ops: &mut Vec<Op>) {
        match value {
            Value::Int(i) => ops.push(Op::Val(Primitive::Int(i.val))),
            Value::Float(f) => ops.push(Op::Val(Primitive::Float(f.val))),
            Value::Str(s) => ops.push(Op::Val(Primitive::Str(s.val))),
            Value::Empty(_pos) => ops.push(Op::Val(Primitive::Empty)),
            Value::Boolean(b) => ops.push(Op::Val(Primitive::Bool(b.val))),
            Value::Symbol(s) => ops.push(Op::Sym(s.val)),
            Value::Tuple(_flds) => unimplemented!("Select expression are not implmented yet"),
            Value::List(_els) => unimplemented!("Select expression are not implmented yet"),
        }
    }
}
