use regex::bytes::Regex;

use crate::Record;

pub trait Filter {
    //fn filter(values : &mut Vec<Record>);
    fn evaluate_predicate(&self, r: &impl Record) -> bool;
    fn filter(&self, values: &mut Vec<impl Record>) {
        let mut ins = None;
        for i in 0..values.len() {
            if !self.evaluate_predicate(&values[i]) {
                if ins.is_none() {
                    ins = Some(i);
                }
            } else if let Some(j) = ins {
                ins = Some(j + 1);
                values.swap(i, j);
            }
        }
        if let Some(j) = ins {
            values.truncate(j);
        }
    }
}

pub struct RegexFilter {
    field_type: Option<usize>,
    regex: Regex,
}

impl RegexFilter {
    pub fn new(field_type: Option<usize>, regex: &str) -> RegexFilter {
        RegexFilter {
            field_type: field_type,
            regex: Regex::new(regex).unwrap(),
        }
    }
}

impl Filter for RegexFilter {
    fn evaluate_predicate(&self, r: &impl Record) -> bool {
        for field in r.field_iter(self.field_type) {
            if self.regex.is_match(field.data) {
                return true;
            }
        }
        return false;
    }
}
