use regex::Regex;
use std::cmp::{max, min};
#[derive(Clone, Debug, PartialEq)]
pub struct ItemContext(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Select,
    FromKW,
    Where,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InfixFn {
    Or,
    And,
    MatchOp,
    EqOp,
    Not, // technically not infix but w/e
}

#[derive(Debug, Clone, PartialEq)]
pub enum Punctuation {
    Comma,
    Paren,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexItem<'a> {
    KW(Keyword),
    InfixFunction(InfixFn),
    Punctuation(Punctuation),
    TableRef(&'a str),
    RegexStr(&'a str),
    FieldRef(Option<&'a str>, Option<&'a str>, Option<&'a str>),
}

pub fn extract_regex_str(input: &str) -> Result<(usize, &str), ()> {
    assert!(input.starts_with('\''));
    let mut escaped = false;
    for (i, c) in input[1..].chars().enumerate() {
        match c {
            '\\' => {
                escaped = !escaped;
            }
            '\'' => {
                if !escaped {
                    return Ok((i + 2, &input[1..i + 1]));
                }
                escaped = false;
            }
            _ => {
                escaped = false;
            }
        }
    }
    Err(())
}

pub fn lex(input: &str) -> Result<Vec<(ItemContext, LexItem)>, String> {
    // matching a set of regexes is not the most efficient way to do this
    // but our users probably won't provide kilobytes of expr-code

    // CAREFUL: the order matters (identifiers are a catch-all) and the /continue/ in the body also matters
    let whitespace = Regex::new(r"^ *").unwrap();
    let keyword_regexes: Vec<regex::Regex> = [r"^select", r"^from", r"^where"]
        .iter()
        .map(|x| Regex::new(x).unwrap())
        .collect();
    let infix_regexes: Vec<regex::Regex> = [r"^or", r"^and", r"^~", r"^=", r"^not"]
        .iter()
        .map(|x| Regex::new(x).unwrap())
        .collect();
    let punctuation_regexes: Vec<regex::Regex> = [r"^,", r"^[)(]"]
        .iter()
        .map(|x| Regex::new(x).unwrap())
        .collect();
    // we only use a regex to find the start of a regex
    // and then manually extract until the first unescaped '
    // I think you can write a regex to do the same thing, but it seems
    // quite complicated.
    let regexstr_regex = Regex::new(r"^'").unwrap();
    // a field ref is a record type (opt.)
    // followed by a field type
    // followed by a subfield type (opt.)
    let field_ref_regex = Regex::new(r"^(([a\*])\.)?([0-9]+|\*)\.?([a-z\*])?").unwrap();
    let table_ref_regex = Regex::new(r"^[a-zA-Z0-9_-]+").unwrap();
    let mut i = 0;
    let mut result = Vec::new();
    'outer: while i < input.len() {
        let cur_i = i;
        dbg!(&result);
        if let Some(cap) = whitespace.captures(&input[i..]) {
            i += cap.get(0).unwrap().end();
        }
        for (j, regex) in keyword_regexes.iter().enumerate() {
            if let Some(cap) = regex.captures(&input[i..]) {
                result.push((
                    ItemContext(i),
                    LexItem::KW([Keyword::Select, Keyword::FromKW, Keyword::Where][j].clone()),
                ));
                i += cap.get(0).unwrap().end();
                continue 'outer;
            }
        }
        for (j, regex) in infix_regexes.iter().enumerate() {
            if let Some(cap) = regex.captures(&input[i..]) {
                result.push((
                    ItemContext(i),
                    LexItem::InfixFunction(
                        [
                            InfixFn::Or,
                            InfixFn::And,
                            InfixFn::MatchOp,
                            InfixFn::EqOp,
                            InfixFn::Not,
                        ][j]
                            .clone(),
                    ),
                ));
                i += cap.get(0).unwrap().end();
                continue 'outer;
            }
        }
        for (j, regex) in punctuation_regexes.iter().enumerate() {
            if let Some(cap) = regex.captures(&input[i..]) {
                result.push((
                    ItemContext(i),
                    LexItem::Punctuation([Punctuation::Comma, Punctuation::Paren][j].clone()),
                ));
                i += cap.get(0).unwrap().end();
                continue 'outer;
            }
        }
        if let Some(cap) = regexstr_regex.captures(&input[i..]) {
            if let Ok((end, slice)) = extract_regex_str(&input[i..]) {
                result.push((ItemContext(i), LexItem::RegexStr(slice)));
                i += end;
                continue 'outer;
            } else {
                return Err(format!("reached end of input while looking for matching ' for the ' at position {}, {}", i,
                  &input[max(0, i)..min(i + 5, input.len())]));
            }
        }
        if let Some(cap) = field_ref_regex.captures(&input[i..]) {
            let record_type = cap.get(2).map(|x| x.as_str());
            let field_type = cap.get(3).map(|x| x.as_str());
            let subfield_type = cap.get(4).map(|x| x.as_str());
            result.push((
                ItemContext(i),
                LexItem::FieldRef(record_type, field_type, subfield_type),
            ));
            i += cap.get(0).unwrap().end();
            continue 'outer;
        }
        if let Some(cap) = table_ref_regex.captures(&input[i..]) {
            result.push((
                ItemContext(i),
                LexItem::Identifier(cap.get(0).map(|x| x.as_str()).unwrap()),
            ));
            i += cap.get(0).unwrap().end();
            continue 'outer;
        }
        if i <= cur_i {
            // no regex matched or we coded a bug
            return Err(format!(
                "Unrecognized token at position {}, '{}'",
                i,
                &input[i..]
            ));
        }
    }
    Ok(result)
}

mod tests {
    use crate::parser::*;
    #[test]
    fn test_extract_regex_str() -> Result<(), ()> {
        let (i1, str1) = extract_regex_str("'aoeu'")?;
        assert_eq!(i1, 6);
        assert_eq!(str1, "aoeu");
        let (i1, str1) = extract_regex_str(r"'ao\'eu'")?;
        assert_eq!(i1, 8);
        assert_eq!(str1, r"ao\'eu");
        let (i1, str1) = extract_regex_str(r"'ao\eu'")?;
        assert_eq!(i1, 7);
        assert_eq!(str1, r"ao\eu");
        let (i1, str1) = extract_regex_str(r"'ao\\'eu'")?;
        assert_eq!(i1, 6);
        assert_eq!(str1, r"ao\\");
        let (i1, str1) = extract_regex_str(r"'ao\\\'eu'")?;
        assert_eq!(i1, 10);
        assert_eq!(str1, r"ao\\\'eu");
        Ok(())
    }

    #[test]
    fn test_tokenize() -> Result<(), ()> {
        let input1 = "  or  and  ~  'aoeu'a.123.b)()123.b123  select , from some_table where  =";
        let r1 = lex(input1);
        dbg!(&r1);
        if let Ok(tokens) = r1 {
            assert_eq!(tokens.len(), 16);
            assert_eq!(
                tokens,
                vec![
                    (ItemContext(2), LexItem::InfixFunction(InfixFn::Or)),
                    (ItemContext(6), LexItem::InfixFunction(InfixFn::And)),
                    (ItemContext(11), LexItem::InfixFunction(InfixFn::MatchOp)),
                    (ItemContext(14), LexItem::RegexStr("aoeu")),
                    (
                        ItemContext(20),
                        LexItem::FieldRef(Some("a"), Some("123"), Some("b"))
                    ),
                    (ItemContext(27), LexItem::Punctuation(Punctuation::Paren)),
                    (ItemContext(28), LexItem::Punctuation(Punctuation::Paren)),
                    (ItemContext(29), LexItem::Punctuation(Punctuation::Paren)),
                    (
                        ItemContext(30),
                        LexItem::FieldRef(None, Some("123"), Some("b"))
                    ),
                    (ItemContext(35), LexItem::FieldRef(None, Some("123"), None)),
                    (ItemContext(40), LexItem::KW(Keyword::Select)),
                    (ItemContext(47), LexItem::Punctuation(Punctuation::Comma)),
                    (ItemContext(49), LexItem::KW(Keyword::FromKW)),
                    (ItemContext(54), LexItem::Identifier("some_table")),
                    (ItemContext(65), LexItem::KW(Keyword::Where)),
                    (ItemContext(72), LexItem::InfixFunction(InfixFn::EqOp)),
                ]
            );
            Ok(())
        } else {
            Err(())
        }
    }

    #[test]
    fn test_tokenize1() -> Result<(), ()> {
        let input1 = "a.123.b";
        let r1 = lex(input1);
        dbg!(&r1);
        if let Ok(tokens) = r1 {
            assert_eq!(tokens.len(), 1);
            Ok(())
        } else {
            Err(())
        }
    }
}
