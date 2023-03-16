/*
EXPR -> OR or EXPR | OR
OR -> TERM and OR  | TERM
TERM -> NOT ~ 'regexstr' | NOT = TERM | NOT
NOT -> not expr | field_ref | ( expr )

x and y or z -> (x and y) or z
*/
use crate::parser::*;

pub fn parse_expr<'a>(
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
            let (rhs, rhs_offset) = parse_expr(input, next_offset + 1)?;
            or_expr.children.push(rhs);
            Ok((or_expr, rhs_offset))
        }
        _ => {
            // just the OR production
            Ok((lhs, next_offset))
        }
    }
}

fn parse_expr_inner<'a>(
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
            let (rhs, rhs_offset) = parse_expr_inner(input, next_offset + 1)?;
            or_expr.children.push(rhs);
            Ok((or_expr, rhs_offset))
        }
        _ => {
            // just the OR production
            Ok((lhs, next_offset))
        }
    }
}

fn parse_NOT<'a>(
    input: &[(ItemContext, LexItem<'a>)],
    offset: usize,
) -> Result<(ParseNode<'a>, usize), String> {
    match input.get(offset) {
        Some((ctx, LexItem::Not)) => {
            let mut not_expr = ParseNode::new(LexItem::Not, ctx.clone());
            let (rhs, rhs_offset) = parse_expr_inner(input, offset + 1)?;
            not_expr.children.push(rhs);
            Ok((not_expr, rhs_offset))
        }
        Some((ctx, LexItem::Paren)) => {
            let (expr, next_offset) = parse_expr_inner(input, offset + 1)?;
            if let Some((_, LexItem::Paren)) = input.get(next_offset) {
                Ok((expr, next_offset + 1))
            } else {
                Err(format!("Mismatched parenthesis. {:?}", ctx))
            }
        }
        Some((ctx, LexItem::FieldRef(record_type, field_type, subfield_type))) => Ok((
            ParseNode::new(
                LexItem::FieldRef(*record_type, *field_type, *subfield_type),
                ctx.clone(),
            ),
            offset + 1,
        )),
        Some((ctx, i)) => Err(format!(
            "Expected 'not' or '(' but found {:?} at {:?}",
            i, ctx
        )),
        _ => Err("Expected 'not' or '(' but reached end of input".to_string()),
    }
}

fn parse_TERM<'a>(
    input: &[(ItemContext, LexItem<'a>)],
    offset: usize,
) -> Result<(ParseNode<'a>, usize), String> {
    let (lhs, next_offset) = parse_NOT(input, offset)?;
    match input.get(next_offset) {
        Some((ctx, LexItem::EqOp)) => {
            let (rhs, next_offset) = parse_NOT(input, next_offset + 1)?;
            let mut eqnode = ParseNode::new(LexItem::EqOp, ctx.clone());
            eqnode.children.push(lhs);
            eqnode.children.push(rhs);
            Ok((eqnode, next_offset))
        }
        Some((ctx, LexItem::MatchOp)) => match (lhs.entry, input.get(next_offset + 1)) {
            (
                LexItem::FieldRef(record_type, field_type, subfield_type),
                Some((ctx2, LexItem::RegexStr(regex))),
            ) => {
                let field_ref_node = ParseNode::new(
                    LexItem::FieldRef(record_type, field_type, subfield_type),
                    ctx.clone(),
                );
                let mut matchnode = ParseNode::new(LexItem::MatchOp, ctx.clone());
                matchnode.children.push(field_ref_node);
                matchnode
                    .children
                    .push(ParseNode::new(LexItem::RegexStr(*regex), ctx2.clone()));
                Ok((matchnode, next_offset + 2))
            }
            _ => Err("todo nice message".to_string()),
        },
        None => Ok((lhs, next_offset)),
        _ => Err("expected = or ~".to_string()),
    }
}

fn parse_COLUMN_EXPR<'a>(
    input: &[(ItemContext, LexItem<'a>)],
    offset: usize,
) -> Result<(ParseNode<'a>, usize), String> {
    if let Some((ctx, LexItem::FieldRef(record_type, field_type, subfield_type))) =
        input.get(offset)
    {
        Ok((
            ParseNode::new(
                LexItem::FieldRef(*record_type, *field_type, *subfield_type),
                ctx.clone(),
            ),
            offset + 1,
        ))
    } else {
        Err("expected a field ref expression".to_string())
    }
}

pub fn parse_OR<'a>(
    input: &[(ItemContext, LexItem<'a>)],
    offset: usize,
) -> Result<(ParseNode<'a>, usize), String> {
    let (lhs, next_offset) = parse_TERM(input, offset)?;
    let c = input.get(next_offset);
    match c {
        Some((context, LexItem::And)) => {
            let mut and_expr = ParseNode::new(LexItem::And, context.clone());
            and_expr.children.push(lhs);
            let (rhs, rhs_offset) = parse_OR(input, next_offset + 1)?;
            and_expr.children.push(rhs);
            Ok((and_expr, rhs_offset))
        }
        _ => Ok((lhs, next_offset)),
    }
}

#[cfg(test)]
mod test {
    use crate::exprparse::*;
    use crate::parser::*;
    #[test]
    fn test_parse1() -> Result<(), String> {
        let str1 = "150 ~ 'aoeu'";
        let (p, _) = parse_expr(&lex(str1)?, 0)?;
        assert_eq!(p.entry, LexItem::MatchOp);
        assert_eq!(p.children.len(), 2);
        assert_eq!(
            p.children[0].entry,
            LexItem::FieldRef(None, Some("150"), None)
        );
        assert_eq!(p.children[1].entry, LexItem::RegexStr("aoeu"));
        Ok(())
    }

    #[test]
    fn test_parse2() -> Result<(), String> {
        let str1 = "not  150 ~ 'aoeu'";
        let (p, _) = parse_expr(&lex(str1)?, 0)?;
        assert_eq!(p.entry, LexItem::Not);
        assert_eq!(p.children.len(), 1);

        let str1 = "not (150 ~ 'aoeu')";
        let (p2, _) = parse_expr(&lex(str1)?, 0)?;
        assert_eq!(p2.entry, LexItem::Not);
        assert_eq!(p2.children.len(), 1);

        assert_eq!(p, p2);
        Ok(())
    }

    #[test]
    fn test_parse3() -> Result<(), String> {
        let str1 = "150 ~ 'aoeu' and 151 ~ 'bcd'";
        let (p, _) = parse_expr(&lex(str1)?, 0)?;
        {
            let mut v: Vec<LexItem<'static>> = Vec::new();
            let mut visitor = |n: &ParseNode<'static>| {
                let e = n.entry.clone();
                v.push(e);
            };
            p.visit_pre(&mut visitor);
            assert_eq!(
                v,
                vec![
                    LexItem::And,
                    LexItem::MatchOp,
                    LexItem::FieldRef(None, Some("150"), None),
                    LexItem::RegexStr("aoeu"),
                    LexItem::MatchOp,
                    LexItem::FieldRef(None, Some("151"), None),
                    LexItem::RegexStr("bcd")
                ]
            );
        }
        {
            let mut v: Vec<LexItem<'static>> = Vec::new();
            let mut visitor = |n: &ParseNode<'static>| {
                let e = n.entry.clone();
                v.push(e);
            };
            p.visit_post(&mut visitor);
            assert_eq!(
                v,
                vec![
                    LexItem::FieldRef(None, Some("150"), None),
                    LexItem::RegexStr("aoeu"),
                    LexItem::MatchOp,
                    LexItem::FieldRef(None, Some("151"), None),
                    LexItem::RegexStr("bcd"),
                    LexItem::MatchOp,
                    LexItem::And
                ]
            );
        }
        Ok(())
    }

    #[test]
    fn test_parse4() -> Result<(), String> {
        let str1 = "150 ~ 'aoeu' and 151 ~ 'bcd' and 152 ~ 'efg'";
        let (p, _) = parse_expr(&lex(str1)?, 0)?;
        {
            let mut v: Vec<LexItem<'static>> = Vec::new();
            let mut visitor = |n: &ParseNode<'static>| {
                let e = n.entry.clone();
                v.push(e);
            };
            p.visit_pre(&mut visitor);
            assert_eq!(
                v,
                vec![
                    LexItem::And,
                    LexItem::MatchOp,
                    LexItem::FieldRef(None, Some("150"), None),
                    LexItem::RegexStr("aoeu"),
                    LexItem::And,
                    LexItem::MatchOp,
                    LexItem::FieldRef(None, Some("151"), None),
                    LexItem::RegexStr("bcd"),
                    LexItem::MatchOp,
                    LexItem::FieldRef(None, Some("152"), None),
                    LexItem::RegexStr("efg")
                ]
            );
        }
        {
            let mut v: Vec<LexItem<'static>> = Vec::new();
            let mut visitor = |n: &ParseNode<'static>| {
                let e = n.entry.clone();
                v.push(e);
            };
            p.visit_post(&mut visitor);
            assert_eq!(
                v,
                vec![
                    LexItem::FieldRef(None, Some("150"), None),
                    LexItem::RegexStr("aoeu"),
                    LexItem::MatchOp,
                    LexItem::FieldRef(None, Some("151"), None),
                    LexItem::RegexStr("bcd"),
                    LexItem::MatchOp,
                    LexItem::FieldRef(None, Some("152"), None),
                    LexItem::RegexStr("efg"),
                    LexItem::MatchOp,
                    LexItem::And,
                    LexItem::And
                ]
            );
        }
        Ok(())
    }
}
