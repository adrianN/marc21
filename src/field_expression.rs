use crate::record::{Record, RecordField, RecordType};

pub trait FieldExpression {
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

impl FieldExpression for FieldTypeSelect {
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

impl FieldRefExpr {
    pub fn new(
        record_type: Option<&str>,
        field_type: Option<&str>,
        subfield_type: Option<&str>,
    ) -> FieldRefExpr {
        FieldRefExpr {
            record_type: record_type.and_then(RecordType::from_str),
            field_type: field_type.and_then(|x| x.parse::<usize>().ok()),
            subfield_type: subfield_type.map(|x| x.bytes().next().unwrap()),
        }
    }
}

impl FieldExpression for FieldRefExpr {
    fn compute<'a>(
        &self,
        record: &'a dyn Record,
    ) -> Box<dyn Iterator<Item = RecordField<'a>> + 'a> {
        if self
            .record_type
            .as_ref()
            .map(|x| *x == record.record_type())
            .unwrap_or(false)
        {
            return Box::new(EmptyIter { _p: PhantomData });
        }
        Box::new(record.field_iter(self.field_type))
    }
}
