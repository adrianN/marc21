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
    parse_SELECT(&tokens, 0).and_then(|(n, i)| {
        if i == tokens.len() {
            Ok(n)
        } else {
            Err(format!(
                "Expected end of input, found {:?} at {}",
                tokens.get(i),
                i
            ))
        }
    })
}

fn parse_SELECT<'a>(
    input: &[(ItemContext, LexItem<'a>)],
    offset: usize,
) -> Result<(ParseNode<'a>, usize), String> {
    let c = input.get(offset);
    match c {
        Some((context, LexItem::KW(Keyword::Select))) => {
            let mut select_clause = ParseNode::new(LexItem::KW(Keyword::Select), context.clone());
            // parse projection list
            let mut next_offset = offset + 1;
            'the_loop: loop {
                if let Some((context, LexItem::FieldRef(record_type, field_type, subfield_type))) =
                    input.get(next_offset)
                {
                    let fieldref_node = ParseNode::new(
                        LexItem::FieldRef(*record_type, *field_type, *subfield_type),
                        context.clone(),
                    );
                    select_clause.children.push(fieldref_node);
                    next_offset += 1;
                    match input.get(next_offset) {
                        Some((context, LexItem::Punctuation(Punctuation::Comma))) => {
                            next_offset += 1;
                        }
                        Some((context, LexItem::KW(Keyword::FromKW))) => {
                            next_offset += 1; // skip the from so we're at the talbe ref after the loop
                            break 'the_loop;
                        }
                        _ => {
                            return Err("expected comma or from".to_string());
                        }
                    }
                } else {
                    return Err("expectod a field ref".to_string());
                }
            }
            // now we should be at the table ref
            if let Some((context, LexItem::Identifier(table_name))) = input.get(next_offset) {
                let table_ref_node =
                    ParseNode::new(LexItem::Identifier(table_name), context.clone());
                select_clause.children.push(table_ref_node);
            } else {
                return Err("expected table ref".to_string());
            }
            next_offset += 1;
            // maybe we have a where clause
            if let Some((context, LexItem::KW(Keyword::Where))) = input.get(next_offset) {
                let (filter_node, recurse_offset) = parse_expr(input, next_offset + 1)?;
                next_offset = recurse_offset;
                select_clause.children.push(filter_node);
            }

            Ok((select_clause, next_offset))
        }
        _ => Err("expected a select".to_string()),
    }
}

#[cfg(test)]
mod test {
    use crate::parser::*;

    #[test]
    fn parse_select() -> Result<(), String> {
        let x = parse("select *, a.150.b from some_table ")?;
        assert_eq!(x.entry, LexItem::KW(Keyword::Select));
        assert_eq!(x.children.len(), 3);
        assert_eq!(
            x.children[0].entry,
            LexItem::FieldRef(None, Some("*"), None)
        );
        assert_eq!(
            x.children[1].entry,
            LexItem::FieldRef(Some("a"), Some("150"), Some("b"))
        );
        assert_eq!(x.children[2].entry, LexItem::Identifier("some_table"));
        Ok(())
    }

    #[test]
    fn parse_where1() -> Result<(), String> {
        let x = parse("select * from some_table where 150 ~ 'aueo'")?;
        assert_eq!(x.entry, LexItem::KW(Keyword::Select));
        assert_eq!(x.children.len(), 3);
        assert_eq!(
            x.children[0].entry,
            LexItem::FieldRef(None, Some("*"), None)
        );
        assert_eq!(x.children[1].entry, LexItem::Identifier("some_table"));
        assert_eq!(
            x.children[2].entry,
            LexItem::InfixFunction(InfixFn::MatchOp)
        );
        let matchop = &x.children[2];
        assert_eq!(matchop.children.len(), 2);
        assert_eq!(
            matchop.children[0].entry,
            LexItem::FieldRef(None, Some("150"), None)
        );
        assert_eq!(matchop.children[1].entry, LexItem::RegexStr("aueo"));
        Ok(())
    }

    #[test]
    fn parse_where2() -> Result<(), String> {
        let x = parse("select * from some_table where 150 = 142")?;
        assert_eq!(x.entry, LexItem::KW(Keyword::Select));
        assert_eq!(x.children.len(), 3);
        assert_eq!(
            x.children[0].entry,
            LexItem::FieldRef(None, Some("*"), None)
        );
        assert_eq!(x.children[1].entry, LexItem::Identifier("some_table"));
        assert_eq!(x.children[2].entry, LexItem::InfixFunction(InfixFn::EqOp));
        let eqop = &x.children[2];
        assert_eq!(eqop.children.len(), 2);
        assert_eq!(
            eqop.children[0].entry,
            LexItem::FieldRef(None, Some("150"), None)
        );
        assert_eq!(
            eqop.children[1].entry,
            LexItem::FieldRef(None, Some("142"), None)
        );
        Ok(())
    }
}
