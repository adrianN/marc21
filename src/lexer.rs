use regex::Regex;
use std::cmp::{max, min};
#[derive(Clone, Debug, PartialEq)]
pub struct ItemContext(pub usize);

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
        r"^(([a\*])\.)?([0-9]+|\*)\.?([a-z\*])?",
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
                    (ItemContext(14), LexItem::RegexStr("aoeu")),
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
