use crate::util::parse_usize;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

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

pub enum RecordType {
    Authority = b'z' as isize,
}

impl<'s> MarcHeader<'s> {
    pub fn record_length(&self) -> usize {
        parse_usize(&self.header[0..5])
    }
    pub fn record_type(&self) -> RecordType {
        match self.header[6] {
            b'z' => RecordType::Authority,
            _ => todo!(),
        }
    }
}

pub struct MarcDirectory<'s> {
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

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn record_length(&self) -> usize {
        self.data.len()
    }
    pub fn directory(&self) -> MarcDirectory<'s> {
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

// Todo we want to be able to iter over this
pub struct MarcRecordBatch<'s> {
    pub records: Vec<MarcRecord<'s>>,
}

pub struct MarcReader<R>
where
    R: Read + Seek,
{
    base_reader: BufReader<R>,
}

impl<R> MarcReader<R>
where
    R: Read + Seek,
{
    // TODO maybe take R instead of BufReader
    pub fn new(reader: BufReader<R>) -> MarcReader<R> {
        MarcReader {
            base_reader: reader,
        }
    }

    pub fn read_batch<'s>(
        &mut self,
        mem: &'s mut [u8],
    ) -> Result<Option<MarcRecordBatch<'s>>, std::io::Error> {
        let mut records: Vec<MarcRecord> = Vec::new();
        let mut i = 0;
        let capacity = mem.len();
        let start_pos = self.base_reader.stream_position().unwrap();
        let read = self.base_reader.read(mem)?;
        if read == 0 {
            return Ok(None);
        }
        while i + 24 < read {
            let header = MarcHeader {
                header: &mem[i..i + 24],
            };
            let record_length = header.record_length();
            assert!(record_length < mem.len());
            if record_length + i <= read {
                // still fits in mem
                records.push(MarcRecord::new(header, &mem[i + 24..i + record_length]));
                i += record_length;
            } else {
                break;
            }
        }
        // mem full, backpedal
        //self.base_reader.seek_relative(-24);
        // TODO seek_relative is unstable in my version of rust
        self.base_reader.seek(SeekFrom::Start(start_pos + i as u64));
        //        let num_bytes = records
        //            .iter()
        //            .map(|r| r.header().record_length())
        //            .sum::<usize>() as u64;
        //        let stream_pos = self.base_reader.stream_position().unwrap();
        //        let bytes_consumed = stream_pos - start_pos;
        //        assert!(bytes_consumed == (num_bytes));

        Ok(Some(MarcRecordBatch { records: records }))
    }
}
