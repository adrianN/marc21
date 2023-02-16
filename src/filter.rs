use crate::field_expression::FieldExpression;
use crate::Record;
use regex::bytes::Regex;
use std::any::Any;

pub trait Filter: Any {
    //fn filter(values : &mut Vec<Record>);
    fn evaluate_predicate(&self, r: &dyn Record) -> bool;
    fn filter<'a>(&self, values: &mut [Box<dyn Record + 'a>]) -> usize {
        // todo Vec::retain?
        let mut ins = None;
        for i in 0..values.len() {
            if !self.evaluate_predicate(&*values[i]) {
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
    fn evaluate_predicate(&self, r: &dyn Record) -> bool {
        for field in self.field_expr.compute(r) {
            if self.regex.is_match(field.data) {
                return true;
            }
        }
        false
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
    fn evaluate_predicate(&self, r: &dyn Record) -> bool {
        for f in &self.children {
            if !f.evaluate_predicate(r) {
                return false;
            }
        }
        true
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
    fn evaluate_predicate(&self, r: &dyn Record) -> bool {
        for f in &self.children {
            if f.evaluate_predicate(r) {
                return true;
            }
        }
        false
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
    fn evaluate_predicate(&self, r: &dyn Record) -> bool {
        !self.child.evaluate_predicate(r)
    }

    fn children(&mut self) -> Option<&mut Vec<Box<dyn Filter>>> {
        None
    }
}
