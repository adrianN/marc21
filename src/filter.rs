use regex::bytes::Regex;

use crate::Record;

pub trait Filter {
    //fn filter(values : &mut Vec<Record>);
    fn evaluate_predicate<'a>(&self, r: &Box<dyn Record + 'a>) -> bool;
    fn filter<'a>(&self, values: &mut [Box<dyn Record + 'a>]) -> usize {
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
        ins.unwrap_or(values.len())
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
    fn evaluate_predicate<'a>(&self, r: &Box<dyn Record + 'a>) -> bool {
        for field in r.field_iter(self.field_type) {
            if self.regex.is_match(field.data) {
                return true;
            }
        }
        return false;
    }
}

pub struct AndFilter {
    children: Vec<Box<dyn Filter>>,
}

impl AndFilter {
    pub fn new(children: Vec<Box<dyn Filter>>) -> AndFilter {
        AndFilter { children: children }
    }
}

impl Filter for AndFilter {
    fn evaluate_predicate<'a>(&self, r: &Box<dyn Record + 'a>) -> bool {
        for f in &self.children {
            if !f.evaluate_predicate(r) {
                return false;
            }
        }
        true
    }
}

pub struct OrFilter {
    children: Vec<Box<dyn Filter>>,
}

impl OrFilter {
    pub fn new(children: Vec<Box<dyn Filter>>) -> OrFilter {
        OrFilter { children: children }
    }
}

impl Filter for OrFilter {
    fn evaluate_predicate<'a>(&self, r: &Box<dyn Record + 'a>) -> bool {
        for f in &self.children {
            if f.evaluate_predicate(r) {
                return true;
            }
        }
        false
    }
}
