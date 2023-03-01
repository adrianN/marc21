/*
STMT -> select COLUMN_EXPR_LIST from TABLE WHERE_CLAUSE
COLUMN_EXPR_LIST -> COLUMN_EXPR | COLUMN_EXPR, COLUMN_EXPR_LIST
WHERE_CLAUSE -> | where EXPR
*/

use crate::exprparse::*;
pub use crate::lexer::*;

pub trait ParseTreeVisitor<'a> {
    fn pre(&mut self, node: &ParseNode<'a>) -> bool;
    fn post(&mut self, node: &ParseNode<'a>) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseNode<'a> {
    pub entry: LexItem<'a>,
    pub context: ItemContext,
    pub children: Vec<ParseNode<'a>>,
}

impl<'a> ParseNode<'a> {
    pub fn new(entry: LexItem<'a>, ctx: ItemContext) -> ParseNode<'a> {
        ParseNode {
            children: Vec::new(),
            entry,
            context: ctx,
        }
    }

    pub fn visit_pre<F>(&self, visitor: &mut F)
    where
        F: FnMut(&ParseNode<'a>),
    {
        visitor(self);
        for c in &self.children {
            c.visit_pre(visitor);
        }
    }

    pub fn visit_post<F>(&self, visitor: &mut F)
    where
        F: FnMut(&ParseNode<'a>),
    {
        for c in &self.children {
            c.visit_post(visitor);
        }
        visitor(self);
    }

    pub fn visit(&self, visitor: &mut impl ParseTreeVisitor<'a>) -> bool {
        if !visitor.pre(self) {
            return false;
        }
        for c in &self.children {
            if !c.visit(visitor) {
                return false;
            }
        }
        visitor.post(self)
    }
}

pub fn parse(input: &str) -> Result<ParseNode, String> {
    let tokens = lex(input)?;
    parse_inner(&tokens, 0).and_then(|(n, i)| {
        if i == tokens.len() {
            Ok(n)
        } else {
            Err(format!(
                "Expected end of input, found {:?} at {}",
                tokens[i], i
            ))
        }
    })
}

fn parse_inner<'a>(
    input: &[(ItemContext, LexItem<'a>)],
    offset: usize,
) -> Result<(ParseNode<'a>, usize), String> {
    let (lhs, next_offset) = parse_OR(input, offset)?;
    let c = input.get(next_offset);
    match c {
        Some((context, LexItem::Or)) => {
            // recurse
            let mut or_expr = ParseNode::new(LexItem::Or, context.clone());
            or_expr.children.push(lhs);
            let (rhs, rhs_offset) = parse_inner(input, next_offset + 1)?;
            or_expr.children.push(rhs);
            Ok((or_expr, rhs_offset))
        }
        _ => {
            // just the OR production
            Ok((lhs, next_offset))
        }
    }
}
