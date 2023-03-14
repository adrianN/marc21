use crate::field_expression::FieldExpression;
use crate::util::TriStateBool;
use crate::Record;
use regex::bytes::Regex;
use std::any::Any;

pub trait Filter: Any {
    //fn filter(values : &mut Vec<Record>);
    fn evaluate_predicate(&self, r: &dyn Record) -> TriStateBool;
    fn filter<'a>(&self, values: &mut [Box<dyn Record + 'a>]) -> (usize, usize) {
        let mut true_pos = 0;
        let mut null_pos = 0;
        let mut false_pos = values.len() - 1;
        while null_pos <= false_pos {
            match self.evaluate_predicate(&*values[null_pos]) {
                TriStateBool::True => {
                    values.swap(true_pos, null_pos);
                    true_pos += 1;
                    null_pos += 1;
                }
                TriStateBool::False => {
                    values.swap(null_pos, false_pos);
                    false_pos -= 1;
                }
                TriStateBool::Null => null_pos += 1,
            }
        }
        (true_pos, null_pos)
    }
    fn children(&mut self) -> Option<&mut Vec<Box<dyn Filter>>>;
}

pub struct RegexFilter {
    field_expr: Box<dyn FieldExpression>,
    regex: Regex,
}

impl RegexFilter {
    pub fn new(field_expr: Box<dyn FieldExpression>, regex: &str) -> RegexFilter {
        RegexFilter {
            field_expr,
            regex: Regex::new(regex).unwrap(),
        }
    }
}

impl Filter for RegexFilter {
    fn evaluate_predicate(&self, r: &dyn Record) -> TriStateBool {
        let mut has_field = false;
        for field in self.field_expr.compute(r) {
            has_field = true;
            if self.regex.is_match(field.data) {
                return TriStateBool::True;
            }
        }
        if has_field {
            TriStateBool::False
        } else {
            TriStateBool::Null
        }
    }
    fn children(&mut self) -> Option<&mut Vec<Box<dyn Filter>>> {
        None
    }
}

pub struct AndFilter {
    children: Vec<Box<dyn Filter>>,
}

impl AndFilter {
    pub fn new(children: Vec<Box<dyn Filter>>) -> AndFilter {
        AndFilter { children }
    }
}

impl Filter for AndFilter {
    fn evaluate_predicate(&self, r: &dyn Record) -> TriStateBool {
        let mut just_nulls = true;
        for f in &self.children {
            match f.evaluate_predicate(r) {
                TriStateBool::False => {
                    return TriStateBool::False;
                }
                TriStateBool::True => {
                    just_nulls = false;
                }
                TriStateBool::Null => {}
            }
        }
        if just_nulls {
            TriStateBool::Null
        } else {
            TriStateBool::True
        }
    }
    fn children(&mut self) -> Option<&mut Vec<Box<dyn Filter>>> {
        Some(&mut self.children)
    }
}

pub struct OrFilter {
    pub children: Vec<Box<dyn Filter>>,
}

impl OrFilter {
    pub fn new(children: Vec<Box<dyn Filter>>) -> OrFilter {
        OrFilter { children }
    }
}

impl Filter for OrFilter {
    fn evaluate_predicate(&self, r: &dyn Record) -> TriStateBool {
        let mut has_null = false;
        for f in &self.children {
            match f.evaluate_predicate(r) {
                TriStateBool::True => {
                    return TriStateBool::True;
                }
                TriStateBool::Null => {
                    has_null = true;
                }
                TriStateBool::False => {}
            }
        }
        if has_null {
            TriStateBool::Null
        } else {
            TriStateBool::False
        }
    }
    fn children(&mut self) -> Option<&mut Vec<Box<dyn Filter>>> {
        Some(&mut self.children)
    }
}

pub struct NotFilter {
    child: Box<dyn Filter>,
}

impl NotFilter {
    pub fn new(child: Box<dyn Filter>) -> NotFilter {
        NotFilter { child }
    }
}

impl Filter for NotFilter {
    fn evaluate_predicate(&self, r: &dyn Record) -> TriStateBool {
        !self.child.evaluate_predicate(r)
    }

    fn children(&mut self) -> Option<&mut Vec<Box<dyn Filter>>> {
        None
    }
}

pub enum FilterInput {
    filter(Box<dyn Filter>),
    field_ref(Box<dyn FieldExpression>),
}
pub struct EqFilter {
    left_child: FilterInput,
    right_child: FilterInput,
}

impl Filter for EqFilter {
    fn evaluate_predicate(&self, r: &dyn Record) -> TriStateBool {
        match (&self.left_child, &self.right_child) {
            (FilterInput::filter(f1), FilterInput::filter(f2)) => {
                if f1.evaluate_predicate(r) == f2.evaluate_predicate(r) {
                    TriStateBool::True
                } else {
                    TriStateBool::False
                }
            }
            (FilterInput::field_ref(f1), FilterInput::field_ref(f2)) => {
                let mut has_f1 = false;
                let mut has_f2 = false;
                // TODO hash instead of nested-loop?
                for field in f1.compute(r) {
                    has_f1 = true;
                    for field2 in f2.compute(r) {
                        has_f2 = true;
                        if field.data == field2.data {
                            return TriStateBool::True;
                        }
                    }
                }
                if has_f1 && has_f2 {
                    TriStateBool::False
                } else {
                    TriStateBool::True
                }
            }
            _ => unreachable!(),
        }
    }
    fn children(&mut self) -> Option<&mut Vec<Box<dyn Filter>>> {
        None
    }
}

#[cfg(test)]
mod test {
    use crate::field_expression::*;
    use crate::filter::*;
    use crate::ownedrecord::*;
    use crate::record::*;
    fn test_data() -> Vec<Box<dyn Record>> {
        let mut result: Vec<Box<dyn Record>> = Vec::new();
        for i in 0..2 {
            let mut r = Box::new(OwnedRecord::new());
            r.add_field(OwnedRecordField {
                field_type: 23,
                data: "foo".as_bytes().to_vec(),
            });
            r.add_field(OwnedRecordField {
                field_type: 0,
                data: format!("{}", i).as_bytes().to_vec(),
            });
            result.push(r);
        }
        for i in 2..4 {
            let mut r = Box::new(OwnedRecord::new());
            r.add_field(OwnedRecordField {
                field_type: 23,
                data: "bar".as_bytes().to_vec(),
            });
            r.add_field(OwnedRecordField {
                field_type: 0,
                data: format!("{}", i).as_bytes().to_vec(),
            });
            result.push(r);
        }
        for i in 4..6 {
            let mut r = Box::new(OwnedRecord::new());
            r.add_field(OwnedRecordField {
                field_type: 42,
                data: "baz".as_bytes().to_vec(),
            });
            r.add_field(OwnedRecordField {
                field_type: 0,
                data: format!("{}", i).as_bytes().to_vec(),
            });
            result.push(r);
        }
        result
    }

    struct TestFilter {
        results: Vec<TriStateBool>,
    }

    impl Filter for TestFilter {
        fn children(&mut self) -> Option<&mut Vec<Box<dyn Filter>>> {
            None
        }
        fn evaluate_predicate(&self, r: &dyn Record) -> TriStateBool {
            let i = r
                .field_iter(Some(0))
                .next()
                .map(|x| {
                    std::str::from_utf8(x.data)
                        .unwrap()
                        .parse::<usize>()
                        .unwrap()
                })
                .unwrap();
            self.results[i]
        }
    }

    #[test]
    fn test_filter() {
        let mut data = test_data();
        let mut filter = TestFilter {
            results: vec![
                TriStateBool::False,
                TriStateBool::Null,
                TriStateBool::True,
                TriStateBool::True,
                TriStateBool::False,
                TriStateBool::False,
            ],
        };

        let (a, b) = filter.filter(&mut data);
        assert_eq!(a, 2);
        assert_eq!(b, 3);
    }

    #[test]
    fn test_regex() {
        let field_expr = FieldRefExpr::new(None, Some("23"), None);
        let regex = RegexFilter::new(Box::new(field_expr), "foo");
        let mut data = test_data();
        let (t, n) = regex.filter(&mut data);
        assert_eq!(t, 2);
        for i in 0..t {
            assert_eq!(regex.evaluate_predicate(&*data[i]), TriStateBool::True);
        }
        assert_eq!(n, 4);
        for i in t..n {
            assert_eq!(regex.evaluate_predicate(&*data[i]), TriStateBool::Null);
        }
        for i in n..data.len() {
            assert_eq!(regex.evaluate_predicate(&*data[i]), TriStateBool::False);
        }
        let order: Vec<usize> = data
            .iter()
            .map(|x| {
                x.field_iter(Some(0))
                    .map(|x| {
                        std::str::from_utf8(x.data)
                            .unwrap()
                            .parse::<usize>()
                            .unwrap()
                    })
                    .next()
                    .unwrap()
            })
            .collect();
        assert_eq!(order, vec![0, 1, 5, 4, 3, 2]);
    }
}
