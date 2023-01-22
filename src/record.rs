pub struct RecordField<'s> {
    pub field_type: usize,
    pub data: &'s [u8],
}

impl<'s> RecordField<'s> {
    pub fn utf8_data(&self) -> &str {
        std::str::from_utf8(self.data).unwrap()
    }
}

pub trait Record {
    // todo nightly features might avoid the box
    // https://stackoverflow.com/questions/39482131/is-it-possible-to-use-impl-trait-as-a-functions-return-type-in-a-trait-defini/39490692#39490692
    fn field_iter(&self, field_type: Option<usize>) -> Box<dyn Iterator<Item = RecordField> + '_>;

    fn to_marc21<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()>;
}
