use crate::util::parse_usize;

pub struct MarcHeader<'s> {
    pub header: &'s [u8],
}

pub struct MarcRecord<'s> {
    header: MarcHeader<'s>,
    data: &'s [u8],
}

pub struct MarcRecordEntries<'s> {
    directory: MarcDirectory<'s>,
    record_payload: &'s [u8],
}

impl<'s> MarcHeader<'s> {
    pub fn record_length(&self) -> usize {
        parse_usize(&self.header[0..5])
    }
}

struct MarcDirectory<'s> {
    directory: &'s [u8],
}

#[derive(Debug)]
struct MarcDirectoryEntryRef<'s> {
    entry: &'s [u8],
}

impl<'s> MarcDirectoryEntryRef<'s> {
    pub fn entry_type(&self) -> usize {
        parse_usize(&self.entry[0..3])
    }
    pub fn len(&self) -> usize {
        parse_usize(&self.entry[3..7])
    }
    pub fn start(&self) -> usize {
        parse_usize(&self.entry[7..12])
    }
}

impl<'s> MarcDirectory<'s> {
    fn get_entry(&self, i: usize) -> MarcDirectoryEntryRef {
        MarcDirectoryEntryRef {
            entry: &self.directory[12 * i..12 * (i + 1)],
        }
    }
    fn num_entries(&self) -> usize {
        self.len() / 12
    }
    fn len(&self) -> usize {
        self.directory.len()
    }
}

fn end_of_entry_position(data: &[u8]) -> Option<usize> {
    data.iter().position(|&x| x == b'\x1e')
}

fn end_of_subfield_position(data: &[u8]) -> Option<usize> {
    data.iter().position(|&x| x == b'\x1f')
}

fn get_directory(data: &[u8]) -> MarcDirectory {
    let directory_end = end_of_entry_position(&data[24..]);
    return MarcDirectory {
        directory: &data[24..24 + directory_end.expect("malformed entry")],
    };
}

impl<'s> MarcRecord<'s> {
    pub fn new(h: MarcHeader<'s>, data: &'s [u8]) -> MarcRecord<'s> {
        MarcRecord {
            header: h,
            data: &data,
        }
    }

    pub fn header(&self) -> &MarcHeader<'s> {
        &self.header
    }

    pub fn record_length(&self) -> usize {
        self.data.len()
    }
    fn directory(&self) -> MarcDirectory<'s> {
        get_directory(self.data)
    }

    pub fn entries(&self) -> MarcRecordEntries<'s> {
        let d = self.directory();
        let d_len = d.len();
        MarcRecordEntries {
            directory: d,
            record_payload: &self.data[24 + d_len..],
        }
    }
}
