use crate::ownedrecord::OwnedRecord;
use crate::record::{OwnedRecordField, Record, RecordField, RecordType};

pub trait Expression {
    fn compute<'a>(&self, record: &'a dyn Record)
        -> Box<dyn Iterator<Item = RecordField<'a>> + 'a>;
}

pub struct FieldTypeSelect {
    field_types: Vec<usize>,
}

impl FieldTypeSelect {
    pub fn new(field_types: Vec<usize>) -> FieldTypeSelect {
        //assert!(field_types.is_sorted());
        FieldTypeSelect { field_types }
    }
}

impl Expression for FieldTypeSelect {
    fn compute<'a>(
        &self,
        record: &'a dyn Record,
    ) -> Box<dyn Iterator<Item = RecordField<'a>> + 'a> {
        dbg!("compute");
        Box::new(record.field_iter_vec(&self.field_types))
    }
}

pub struct FieldRefExpr {
    record_type: Option<RecordType>,
    field_type: Option<usize>,
    subfield_type: Option<u8>,
}

use std::marker::PhantomData;
struct EmptyIter<T> {
    _p: PhantomData<T>,
}

impl<T> Iterator for EmptyIter<T>
where
    T: Sized,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl Expression for FieldRefExpr {
    fn compute<'a>(
        &self,
        record: &'a dyn Record,
    ) -> Box<dyn Iterator<Item = RecordField<'a>> + 'a> {
        if self
            .record_type.as_ref()
            .map(|x| *x == record.record_type())
            .unwrap_or(false)
        {
            return Box::new(EmptyIter { _p: PhantomData });
        }
        Box::new(record.field_iter(self.field_type))
    }
}
