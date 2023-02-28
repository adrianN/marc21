use crate::exprparse::*;
use crate::field_expression::*;
use crate::filter::*;
use crate::parser::*;
use std::any::TypeId;

struct TranslationVisitor {
    exprs: Vec<Box<dyn Filter>>,
}

impl TranslationVisitor {
    pub fn new() -> TranslationVisitor {
        TranslationVisitor { exprs: Vec::new() }
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
            LexItem::Or => {
                let second: Box<dyn Filter> = self.exprs.pop().unwrap();
                let first: Box<dyn Filter> = self.exprs.pop().unwrap();
                let mut arguments = Vec::new();
                TranslationVisitor::flatten(first, &mut arguments);
                TranslationVisitor::flatten(second, &mut arguments);
                self.exprs.push(Box::new(OrFilter::new(arguments)));
                /*
                todo it would be nice if code like this compiled
                                                self.exprs.push(Box::new(OrFilter::new(
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
            LexItem::And => {
                let second: Box<dyn Filter> = self.exprs.pop().unwrap();
                let first: Box<dyn Filter> = self.exprs.pop().unwrap();
                let mut arguments = Vec::new();
                TranslationVisitor::flatten(first, &mut arguments);
                TranslationVisitor::flatten(second, &mut arguments);
                self.exprs.push(Box::new(AndFilter::new(arguments)));
                true
            }
            LexItem::MatchOp => {
                let children: Vec<LexItem> =
                    node.children.iter().map(|x| x.entry.clone()).collect();
                assert!(children.len() == 2);
                if let LexItem::FieldRef(record_type, field_type, subfield_type) = children[0] {
                    if let LexItem::RegexStr(regexstr) = children[1] {
                        // todo with the FieldExpr stuff this deserves its own parsing function
                        let field_expr =
                            Box::new(FieldRefExpr::new(record_type, field_type, subfield_type));
                        self.exprs
                            .push(Box::new(RegexFilter::new(field_expr, regexstr)));
                    } else {
                        unreachable!();
                    }
                } else {
                    unreachable!();
                }
                true
            }
            LexItem::Not => {
                let argument = self.exprs.pop().unwrap();
                self.exprs.push(Box::new(NotFilter::new(argument)));
                true
            }
            LexItem::Paren => {
                unreachable!();
            }
            LexItem::RegexStr(_) => true,
            LexItem::FieldRef(_, _, _) => true,
        }
    }
}

pub fn compile(input: &str) -> Result<Box<dyn Filter>, String> {
    let parsetree = parse(input)?;
    let mut visitor = TranslationVisitor::new();
    parsetree.visit(&mut visitor);
    dbg!(visitor.exprs.len());
    assert!(visitor.exprs.len() == 1);
    Ok(visitor.exprs.pop().unwrap())
}
