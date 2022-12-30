use std::env;
use std::fs;
use std::str;

struct MarcHeader<'s> {
    header: &'s [u8],
}

struct MarcRecordRef<'s> {
    header: MarcHeader<'s>,
    data: &'s [u8],
}

fn parse_usize(slice: &[u8]) -> usize {
    let mut n: usize = 0;
    for i in slice {
        n *= 10;
        n += (i - b'0') as usize;
    }
    n
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

fn get_header(data: &[u8]) -> MarcHeader {
    MarcHeader {
        header: &data[0..24],
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

struct MarcRecordEntry<'s> {
    data: &'s [u8],
}

struct MarcRecordEntries<'s> {
    directory: MarcDirectory<'s>,
    record_payload: &'s [u8],
}

impl<'s> MarcRecordRef<'s> {
    pub fn new(data: &[u8]) -> MarcRecordRef {
        let h = get_header(data);
        let h_len = h.record_length();
        MarcRecordRef {
            header: h,
            data: &data[0..h_len],
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

struct MarcRecord {
    data: Vec<u8>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let contents = fs::read(filename).expect("can't read");

    dbg!(contents.len());
    let mut offset = 0;
    while offset < contents.len() {
        let r = MarcRecordRef::new(&contents[offset..]);
        let l = r.header().record_length();
        offset += l;
        assert!(r.header().record_length() == r.record_length());
        //dbg!(str::from_utf8(&r.data));
        if l < 10 {
            println!("0000{}", l);
        } else if l < 100 {
            println!("000{}", l);
        } else if l < 1000 {
            println!("00{}", l);
        } else if l < 10000 {
            println!("0{}", l);
        } else {
            println!("{}", l);
        }
        //      let d = r.directory();
        //      dbg!(str::from_utf8(d.directory));
        //      for i in 0..d.num_entries() {
        //        let d_e = d.get_entry(i);
        //        dbg!(&d_e);
        //      }
    }
}
