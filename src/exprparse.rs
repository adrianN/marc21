/*
EXPR -> OR or EXPR | OR
OR -> TERM and OR  | TERM
TERM -> NOT ~ 'regexstr' | NOT = TERM | NOT
NOT -> IDENTIFIER ( LIST_OF_EXPR ) | field_ref | ( expr )
LIST_OF_EXPR -> expr | expr , LIST_OF_EXPR

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
        Some((context, LexItem::InfixFunction(InfixFn::Or))) => {
            // recurse
            let mut or_expr = ParseNode::new(LexItem::InfixFunction(InfixFn::Or), context.clone());
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
        Some((context, LexItem::InfixFunction(InfixFn::Or))) => {
            // recurse
            let mut or_expr = ParseNode::new(LexItem::InfixFunction(InfixFn::Or), context.clone());
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

fn parse_expr_list<'a>(
    input: &[(ItemContext, LexItem<'a>)],
    offset: usize,
) -> Result<(Vec<ParseNode<'a>>, usize), String> {
    let mut result = Vec::new();
    let mut cur_off = offset;
    loop {
        dbg!(input.get(cur_off));
        let (exp, off) = parse_expr_inner(input, cur_off)?;
        result.push(exp);
        match input.get(off) {
            Some((_, LexItem::Punctuation(Punctuation::Comma))) => {
                cur_off = off + 1;
            }
            x => {
                dbg!(&x);
                cur_off = off;
                break;
            }
        }
    }
    dbg!(&result, cur_off);
    Ok((result, cur_off))
}

fn parse_NOT<'a>(
    input: &[(ItemContext, LexItem<'a>)],
    offset: usize,
) -> Result<(ParseNode<'a>, usize), String> {
    dbg!(input.get(offset));
    match input.get(offset) {
        Some((ctx, LexItem::Identifier(n))) => {
            dbg!(&n);
            if let Some((_, LexItem::Punctuation(Punctuation::Paren))) = input.get(offset + 1) {
                let (children, next_offset) = parse_expr_list(input, offset + 2)?;
                let mut identifier_expr = ParseNode::new(LexItem::Identifier(n), ctx.clone());
                identifier_expr.children = children;
                if let Some((_, LexItem::Punctuation(Punctuation::Paren))) = input.get(next_offset)
                {
                    return Ok((identifier_expr, next_offset + 1));
                } else {
                    return Err(format!(
                        "expected ')' after expr list found {:?}",
                        input.get(next_offset)
                    ));
                }
            } else {
                Err(format!(
                    "expected open paren after identifier, found {:?}",
                    input.get(offset + 1)
                ))
            }
        }
        Some((ctx, LexItem::Punctuation(Punctuation::Paren))) => {
            let (expr, next_offset) = parse_expr_inner(input, offset + 1)?;
            if let Some((_, LexItem::Punctuation(Punctuation::Paren))) = input.get(next_offset) {
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
            "Expected identifier, field ref, or '(' but found {:?} at {:?}",
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
    dbg!(&lhs);
    dbg!(input.get(next_offset));
    match input.get(next_offset) {
        Some((ctx, LexItem::InfixFunction(InfixFn::EqOp))) => {
            let (rhs, next_offset) = parse_NOT(input, next_offset + 1)?;
            let mut eqnode = ParseNode::new(LexItem::InfixFunction(InfixFn::EqOp), ctx.clone());
            eqnode.children.push(lhs);
            eqnode.children.push(rhs);
            Ok((eqnode, next_offset))
        }
        Some((ctx, LexItem::InfixFunction(InfixFn::MatchOp))) => {
            match (lhs.entry, input.get(next_offset + 1)) {
                (
                    LexItem::FieldRef(record_type, field_type, subfield_type),
                    Some((ctx2, LexItem::RegexStr(regex))),
                ) => {
                    let field_ref_node = ParseNode::new(
                        LexItem::FieldRef(record_type, field_type, subfield_type),
                        ctx.clone(),
                    );
                    let mut matchnode =
                        ParseNode::new(LexItem::InfixFunction(InfixFn::MatchOp), ctx.clone());
                    matchnode.children.push(field_ref_node);
                    matchnode
                        .children
                        .push(ParseNode::new(LexItem::RegexStr(*regex), ctx2.clone()));
                    Ok((matchnode, next_offset + 2))
                }
                _ => Err("todo nice message".to_string()),
            }
        }
        _ => Ok((lhs, next_offset)),
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
        Some((context, LexItem::InfixFunction(InfixFn::And))) => {
            let mut and_expr =
                ParseNode::new(LexItem::InfixFunction(InfixFn::And), context.clone());
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
        assert_eq!(p.entry, LexItem::InfixFunction(InfixFn::MatchOp));
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
        let str1 = "not (150 ~ 'aoeu') ";
        let (p, _) = parse_expr(&lex(str1)?, 0)?;
        assert_eq!(p.entry, LexItem::Identifier("not"));
        assert_eq!(p.children.len(), 1);

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
                    LexItem::InfixFunction(InfixFn::And),
                    LexItem::InfixFunction(InfixFn::MatchOp),
                    LexItem::FieldRef(None, Some("150"), None),
                    LexItem::RegexStr("aoeu"),
                    LexItem::InfixFunction(InfixFn::MatchOp),
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
                    LexItem::InfixFunction(InfixFn::MatchOp),
                    LexItem::FieldRef(None, Some("151"), None),
                    LexItem::RegexStr("bcd"),
                    LexItem::InfixFunction(InfixFn::MatchOp),
                    LexItem::InfixFunction(InfixFn::And)
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
                    LexItem::InfixFunction(InfixFn::And),
                    LexItem::InfixFunction(InfixFn::MatchOp),
                    LexItem::FieldRef(None, Some("150"), None),
                    LexItem::RegexStr("aoeu"),
                    LexItem::InfixFunction(InfixFn::And),
                    LexItem::InfixFunction(InfixFn::MatchOp),
                    LexItem::FieldRef(None, Some("151"), None),
                    LexItem::RegexStr("bcd"),
                    LexItem::InfixFunction(InfixFn::MatchOp),
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
                    LexItem::InfixFunction(InfixFn::MatchOp),
                    LexItem::FieldRef(None, Some("151"), None),
                    LexItem::RegexStr("bcd"),
                    LexItem::InfixFunction(InfixFn::MatchOp),
                    LexItem::FieldRef(None, Some("152"), None),
                    LexItem::RegexStr("efg"),
                    LexItem::InfixFunction(InfixFn::MatchOp),
                    LexItem::InfixFunction(InfixFn::And),
                    LexItem::InfixFunction(InfixFn::And)
                ]
            );
        }
        Ok(())
    }
    #[test]
    fn test_parse5() -> Result<(), String> {
        let str = "not_null(150)";
        let (p, _) = parse_expr(&lex(str)?, 0)?;
        assert_eq!(p.entry, LexItem::Identifier("not_null"));
        assert_eq!(
            p.children.get(0).map(|x| x.entry.clone()),
            Some(LexItem::FieldRef(None, Some("150"), None))
        );
        Ok(())
    }
}
