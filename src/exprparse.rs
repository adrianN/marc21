/*
EXPR -> OR or EXPR | OR
OR -> REGEX and OR  | REGEX
REGEX -> fieldref ~ 'regexstr' | NOT
NOT -> not expr | ( expr )

x and y or z -> (x and y) or z
*/

use regex::Regex;
use std::cmp::{max, min};

#[derive(Debug, Clone, PartialEq)]
pub enum LexItem<'a> {
    Or,
    And,
    MatchOp,
    Not,
    Paren,
    RegexStr(&'a str),
    FieldRef(Option<&'a str>, Option<&'a str>, Option<&'a str>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemContext(usize);

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
            entry: entry,
            context: ctx,
        }
    }
}

fn extract_regex_str(input: &str) -> Result<(usize, &str), ()> {
    assert!(input.chars().nth(0) == Some('\''));
    let mut escaped = false;
    for (i, c) in input[1..].chars().enumerate() {
        match c {
            '\\' => {
                escaped = !escaped;
            }
            '\'' => {
                if !escaped {
                    return Ok((i + 2, &input[..i + 2]));
                }
                escaped = false;
            }
            _ => {
                escaped = false;
            }
        }
    }
    return Err(());
}

fn lex<'a>(input: &'a str) -> Result<Vec<(ItemContext, LexItem<'a>)>, String> {
    // matching a set of regexes is not the most efficient way to do this
    // but our users probably won't provide kilobytes of expr-code
    let token_regexes: Vec<regex::Regex> = [
        r"^  *",
        r"^or",
        r"^and",
        r"^~",
        r"^not",
        r"^[)(]",
        // we only use a regex to find the start of a regex
        // and then manually extract until the first unescaped '
        // I think you can write a regex to do the same thing, but it seems
        // quite complicated.
        r"^'",
        // a field ref is a record type (opt.)
        // followed by a field type
        // followed by a subfield type (opt.)
        r"^(([a])\.)?([0-9]+)\.?([a-z])?",
    ]
    .iter()
    .map(|x| Regex::new(x).unwrap())
    .collect();
    assert!(token_regexes.len() == 8);
    let mut i = 0;
    let mut result = Vec::new();
    'outer: while i < input.len() {
        dbg!(&result);
        for (j, regex) in token_regexes.iter().enumerate() {
            if let Some(cap) = regex.captures(&input[i..]) {
                dbg!(&input[i..], j, regex, &cap);
                let cur_i = i;
                match j {
                    0 => {} // skip whitespace
                    1..=5 => {
                        result.push((
                            ItemContext(i),
                            [
                                LexItem::Or,
                                LexItem::And,
                                LexItem::MatchOp,
                                LexItem::Not,
                                LexItem::Paren,
                            ][j - 1]
                                .clone(),
                        ));
                    }
                    6 => {
                        if let Ok((end, slice)) = extract_regex_str(&input[i..]) {
                            result.push((ItemContext(i), LexItem::RegexStr(slice)));
                            i += end - 1;
                        } else {
                            return Err(format!("reached end of input while looking for matching ' for the ' at position {}, {}", i,
                &input[max(0, i - 5)..min(i + 5, input.len())]));
                        }
                    }
                    7 => {
                        let record_type = cap.get(2).map(|x| x.as_str());
                        let field_type = cap.get(3).map(|x| x.as_str());
                        let subfield_type = cap.get(4).map(|x| x.as_str());
                        result.push((
                            ItemContext(i),
                            LexItem::FieldRef(record_type, field_type, subfield_type),
                        ));
                    }
                    _ => {
                        unreachable!()
                    }
                }
                i += cap.get(0).unwrap().end();
                assert!(i > cur_i);
                continue 'outer;
            }
        }
        return Err(format!(
            "Unrecognized token at position {}, '{}'",
            i,
            &input[i..]
        ));
    }
    Ok(result)
}

pub fn parse<'a>(input: &'a str) -> Result<ParseNode<'a>, String> {
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

fn parse_NOT<'a>(
    input: &[(ItemContext, LexItem<'a>)],
    offset: usize,
) -> Result<(ParseNode<'a>, usize), String> {
    match input.get(offset) {
        Some((ctx, LexItem::Not)) => {
            let mut not_expr = ParseNode::new(LexItem::Not, ctx.clone());
            let (rhs, rhs_offset) = parse_inner(input, offset + 1)?;
            not_expr.children.push(rhs);
            Ok((not_expr, rhs_offset))
        }
        Some((ctx, LexItem::Paren)) => {
            let (expr, next_offset) = parse_inner(input, offset + 1)?;
            if let Some((_, LexItem::Paren)) = input.get(next_offset) {
                Ok((expr, next_offset + 1))
            } else {
                Err(format!("Mismatched parenthesis. {:?}", ctx))
            }
        }
        Some((ctx, i)) => Err(format!(
            "Expected 'not' or '(' but found {:?} at {:?}",
            i, ctx
        )),
        _ => Err(format!("Expected 'not' or '(' but reached end of input")),
    }
}

fn parse_REGEX<'a>(
    input: &[(ItemContext, LexItem<'a>)],
    offset: usize,
) -> Result<(ParseNode<'a>, usize), String> {
    if let Ok((not_expr, next_offset)) = parse_NOT(input, offset) {
        Ok((not_expr, next_offset))
    } else {
        match (
            input.get(offset),
            input.get(offset + 1),
            input.get(offset + 2),
        ) {
            (
                Some((ctx, LexItem::FieldRef(record_type, field_type, subfield_type))),
                Some((ctx1, LexItem::MatchOp)),
                Some((ctx2, LexItem::RegexStr(regex))),
            ) => {
                let mut matchnode = ParseNode::new(LexItem::MatchOp, ctx1.clone());
                matchnode.children.push(ParseNode::new(
                    LexItem::FieldRef(*record_type, *field_type, *subfield_type),
                    ctx.clone(),
                ));
                matchnode
                    .children
                    .push(ParseNode::new(LexItem::RegexStr(*regex), ctx2.clone()));
                Ok((matchnode, offset + 3))
            }
            _ => Err("todo nice message".to_string()),
        }
    }
}

fn parse_OR<'a>(
    input: &[(ItemContext, LexItem<'a>)],
    offset: usize,
) -> Result<(ParseNode<'a>, usize), String> {
    let (lhs, next_offset) = parse_REGEX(input, offset)?;
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

#[cfg(test)]
mod tests {
    use crate::exprparse::*;
    #[test]
    fn test_extract_regex_str() -> Result<(), ()> {
        let (i1, str1) = extract_regex_str("'aoeu'")?;
        assert_eq!(i1, 6);
        assert_eq!(str1, "'aoeu'");
        let (i1, str1) = extract_regex_str(r"'ao\'eu'")?;
        assert_eq!(i1, 8);
        assert_eq!(str1, r"'ao\'eu'");
        let (i1, str1) = extract_regex_str(r"'ao\eu'")?;
        assert_eq!(i1, 7);
        assert_eq!(str1, r"'ao\eu'");
        let (i1, str1) = extract_regex_str(r"'ao\\'eu'")?;
        assert_eq!(i1, 6);
        assert_eq!(str1, r"'ao\\'");
        let (i1, str1) = extract_regex_str(r"'ao\\\'eu'")?;
        assert_eq!(i1, 10);
        assert_eq!(str1, r"'ao\\\'eu'");
        Ok(())
    }

    #[test]
    fn test_tokenize() -> Result<(), ()> {
        let input1 = "  or  and  ~  'aoeu'a.123.b)()123.b123  ";
        let r1 = lex(input1);
        dbg!(&r1);
        if let Ok(tokens) = r1 {
            assert_eq!(tokens.len(), 10);
            assert_eq!(
                tokens,
                vec![
                    (ItemContext(2), LexItem::Or),
                    (ItemContext(6), LexItem::And),
                    (ItemContext(11), LexItem::MatchOp),
                    (ItemContext(14), LexItem::RegexStr("\'aoeu\'")),
                    (
                        ItemContext(20),
                        LexItem::FieldRef(Some("a"), Some("123"), Some("b"))
                    ),
                    (ItemContext(27), LexItem::Paren),
                    (ItemContext(28), LexItem::Paren),
                    (ItemContext(29), LexItem::Paren),
                    (
                        ItemContext(30),
                        LexItem::FieldRef(None, Some("123"), Some("b"))
                    ),
                    (ItemContext(35), LexItem::FieldRef(None, Some("123"), None))
                ]
            );
            Ok(())
        } else {
            Err(())
        }
    }

    #[test]
    fn test_parse1() -> Result<(), String> {
        let str1 = "150 ~ 'aoeu'";
        let p = parse(str1)?;
        assert_eq!(p.entry, LexItem::MatchOp);
        assert_eq!(p.children.len(), 2);
        assert_eq!(
            p.children[0].entry,
            LexItem::FieldRef(None, Some("150"), None)
        );
        assert_eq!(p.children[1].entry, LexItem::RegexStr("'aoeu'"));

        let str1 = "not  150 ~ 'aoeu'";
        let p = parse(str1)?;
        assert_eq!(p.entry, LexItem::Not);
        assert_eq!(p.children.len(), 1);

        let str1 = "not (150 ~ 'aoeu')";
        let p2 = parse(str1)?;
        assert_eq!(p2.entry, LexItem::Not);
        assert_eq!(p2.children.len(), 1);

        assert_eq!(p, p2);

        let str1 = "150 ~ 'aoeu' and 150 ~ 'bcd'";
        let p = parse(str1);
        /*
        Ok(ParseNode { entry: And, context: ItemContext(13), children: [ParseNode { entry: MatchOp, context: ItemContext(4), children: [ParseNode { entry: FieldRef(None, Some("150"), None), context: ItemContext(0), children: [] }, ParseNode { entry: RegexStr("\'aoeu\'"), context: ItemContext(6), children: [] }] },
        ParseNode { entry: MatchOp, context: ItemContext(21), children: [ParseNode { entry: FieldRef(None, Some("150"), None), context: ItemContext(17), children: [] }, ParseNode { entry: RegexStr("\'bcd\'"), context: ItemContext(23), children: [] }] }] })`,
        */

        assert!(p.is_ok());
        Ok(())
    }
}
