use crate::record::*;
pub struct OwnedRecord {
    pub header: [u8; 24],
    pub field_types: Vec<usize>,
    pub field_data: Vec<Vec<u8>>,
}

impl OwnedRecord {
    pub fn new() -> OwnedRecord {
        OwnedRecord {
            header: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            field_types: Vec::new(),
            field_data: Vec::new(),
        }
    }

    pub fn add_field(&mut self, field: OwnedRecordField) {
        self.field_types.push(field.field_type);
        self.field_data.push(field.data);
    }

    pub fn add_field_from_iter(&mut self, field_iter: &mut dyn Iterator<Item = RecordField>) {
        for field in field_iter {
            dbg!("add_field_from_iter");
            self.add_field(field.to_owned());
        }
        self.update_len();
    }

    fn update_len(&mut self) {
        let data_len: usize = self.field_data.iter().map(|x| x.len()).sum();
        let dict_len: usize = 12 * self.field_data.len();
        let mut l: usize = data_len + dict_len + self.header.len();
        for i in 0..5 {
            self.header[4 - i] = '0' as u8 + (l % 10) as u8;
            l = l / 10;
        }
    }
}

struct OwnedRecordFieldIter<'s> {
    i: usize,
    field_types: Vec<usize>,
    record: &'s OwnedRecord,
}

impl<'s> Iterator for OwnedRecordFieldIter<'s> {
    type Item = RecordField<'s>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.record.field_types.len() {
            let idx = self.i;
            self.i += 1;
            let field_type = self.record.field_types[idx];
            if self.field_types.binary_search(&field_type).is_ok() || self.field_types.len() == 0 {
                let field_data = &self.record.field_data[idx];
                return Some(RecordField {
                    field_type,
                    data: field_data,
                });
            }
        }
        None
    }
}

impl Record for OwnedRecord {
    fn record_type(&self) -> RecordType {
        todo!();
    }
    fn field_iter_vec(
        &self,
        field_types: &Vec<usize>,
    ) -> Box<dyn Iterator<Item = RecordField> + '_> {
        Box::new(OwnedRecordFieldIter {
            i: 0,
            field_types: field_types.clone(),
            record: &self,
        })
    }

    fn to_owned(self) -> OwnedRecord {
        self
    }
    fn field_iter(&self, field_types: Option<usize>) -> Box<dyn Iterator<Item = RecordField> + '_> {
        // todo we probably don't want to alloc a vec here
        if let Some(x) = field_types {
            self.field_iter_vec(&vec![x])
        } else {
            self.field_iter_vec(&Vec::new())
        }
    }
    fn to_marc21(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        todo!()
    }
}
