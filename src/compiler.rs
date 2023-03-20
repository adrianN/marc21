use crate::exprparse::*;
use crate::field_expression::*;
use crate::filter::*;
use crate::parser::*;
use crate::projection::*;
use std::any::TypeId;

struct TranslationVisitor {
    filter_exprs: Vec<Box<dyn Filter>>,
    field_exprs: Vec<Box<dyn FieldExpression>>,
    projection_exprs: Vec<Box<dyn FieldExpression>>,
    table_name: String,
}

impl TranslationVisitor {
    pub fn new() -> TranslationVisitor {
        TranslationVisitor {
            filter_exprs: Vec::new(),
            field_exprs: Vec::new(),
            projection_exprs: Vec::new(),
            table_name: "".to_string(),
        }
    }

    fn flatten(mut x: Box<dyn Filter>, arguments: &mut Vec<Box<dyn Filter>>) {
        if x.type_id() == TypeId::of::<OrFilter>() || x.type_id() == TypeId::of::<AndFilter>() {
            arguments.extend(std::mem::take(x.children().unwrap()));
        } else {
            arguments.push(x);
        }
    }
}

impl<'a> ParseTreeVisitor<'a> for TranslationVisitor {
    fn pre(&mut self, _node: &ParseNode) -> bool {
        true
    }

    fn post(&mut self, node: &ParseNode) -> bool {
        match node.entry {
            LexItem::KW(Keyword::Select) => {
                // FieldRef children of select are projections
                for c in &node.children {
                    if let LexItem::FieldRef(r, f, s) = c.entry {
                        self.projection_exprs
                            .push(Box::new(FieldRefExpr::new(r, f, s)));
                    }
                }
                true
            }
            LexItem::Punctuation(Punctuation::Comma) => {
                unreachable!()
            }
            LexItem::KW(Keyword::FromKW) => {
                unreachable!()
            }
            LexItem::Identifier(table_name) => {
                self.table_name = table_name.to_string();
                true
            }
            LexItem::KW(Keyword::Where) => {
                unreachable!()
            }
            LexItem::InfixFunction(InfixFn::Or) => {
                let second: Box<dyn Filter> = self.filter_exprs.pop().unwrap();
                let first: Box<dyn Filter> = self.filter_exprs.pop().unwrap();
                let mut arguments = Vec::new();
                TranslationVisitor::flatten(first, &mut arguments);
                TranslationVisitor::flatten(second, &mut arguments);
                self.filter_exprs.push(Box::new(OrFilter::new(arguments)));
                /*
                todo it would be nice if code like this compiled
                                                self.filter_exprs.push(Box::new(OrFilter::new(
                                                    [first, second]
                                                        .iter_mut()
                                                        .flat_map(|x: Box<dyn Filter>| {
                                                            if x.type_id() == TypeId::of::<OrFilter>() {
                                                                std::mem::take(x.children().unwrap()).into_iter()
                                                            } else {
                                                                vec![x].into_iter()
                                                            }
                                                        })
                                                        .collect(),
                                                )));
                                                true
                */
                true
            }
            LexItem::InfixFunction(InfixFn::And) => {
                let second: Box<dyn Filter> = self.filter_exprs.pop().unwrap();
                let first: Box<dyn Filter> = self.filter_exprs.pop().unwrap();
                let mut arguments = Vec::new();
                TranslationVisitor::flatten(first, &mut arguments);
                TranslationVisitor::flatten(second, &mut arguments);
                self.filter_exprs.push(Box::new(AndFilter::new(arguments)));
                true
            }
            LexItem::InfixFunction(InfixFn::MatchOp) => {
                let children: Vec<LexItem> =
                    node.children.iter().map(|x| x.entry.clone()).collect();
                assert!(children.len() == 2);

                if let Some(field_expr) = self.field_exprs.pop() {
                    if let LexItem::RegexStr(regexstr) = children[1] {
                        self.filter_exprs
                            .push(Box::new(RegexFilter::new(field_expr, regexstr)));
                    } else {
                        unreachable!();
                    }
                } else {
                    unreachable!();
                }
                true
            }
            LexItem::InfixFunction(InfixFn::EqOp) => {
                assert!(self.filter_exprs.len() + self.field_exprs.len() == 2);
                let mut inps = Vec::new();
                for filter_expr in self.filter_exprs.drain(0..) {
                    inps.push(FilterInput::filter(filter_expr));
                }
                for field_ref in self.field_exprs.drain(0..) {
                    inps.push(FilterInput::field_ref(field_ref));
                }
                let rhs = inps.pop().unwrap();
                let lhs = inps.pop().unwrap();
                self.filter_exprs.push(Box::new(EqFilter::new(lhs, rhs)));

                true
            }
            LexItem::Identifier("not") => {
                let argument = self.filter_exprs.pop().unwrap();
                self.filter_exprs.push(Box::new(NotFilter::new(argument)));
                true
            }
            LexItem::Punctuation(Punctuation::Paren) => {
                unreachable!();
            }
            LexItem::RegexStr(_) => true,
            LexItem::FieldRef(record_type, field_type, subfield_type) => {
                self.field_exprs.push(Box::new(FieldRefExpr::new(
                    record_type,
                    field_type,
                    subfield_type,
                )));

                true
            }
        }
    }
}

pub struct CompilationResult {
    pub projection: Projection,
    pub filter_expr: Option<Box<dyn Filter>>,
    pub table_name: String,
}

pub fn compile(input: &str) -> Result<CompilationResult, String> {
    let parsetree = parse(input)?;
    let mut visitor = TranslationVisitor::new();
    parsetree.visit(&mut visitor);
    dbg!(visitor.filter_exprs.len());
    assert!(visitor.filter_exprs.len() <= 1);
    Ok(CompilationResult {
        projection: Projection::new(visitor.projection_exprs),
        filter_expr: visitor.filter_exprs.pop(),
        table_name: visitor.table_name,
    })
}
