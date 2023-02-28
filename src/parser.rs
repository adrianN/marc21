/*
STMT -> select COLUMN_EXPR_LIST from TABLE WHERE_CLAUSE
COLUMN_EXPR_LIST -> COLUMN_EXPR | COLUMN_EXPR, COLUMN_EXPR_LIST
WHERE_CLAUSE -> | where EXPR
*/

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
