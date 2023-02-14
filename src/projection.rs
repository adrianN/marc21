use crate::expression::Expression;
use crate::ownedrecord::OwnedRecord;
use crate::parsedrecord::ParsedRecord;
use crate::Record;

pub struct Projection {
    exprs: Vec<Box<dyn Expression>>,
}

impl Projection {
    pub fn new(exprs: Vec<Box<dyn Expression>>) -> Projection {
        Projection { exprs }
    }
    pub fn project<'a>(&self, values: &mut [Box<dyn Record + 'a>]) -> usize {
        for i in 0..values.len() {
            // todo this loses header information
            let mut result = OwnedRecord::new();
            for expr in &self.exprs {
                // todo why doesn't for record in values { compute(&*record) } work?
                result.add_field_from_iter(&mut expr.compute(&*values[i]));
            }
            values[i] = Box::new(result);
        }
        0
    }
}
