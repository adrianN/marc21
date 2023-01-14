use crate::util::parse_usize;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

#[derive(Debug)]
pub struct MarcHeader<'s> {
    pub header: &'s [u8],
}

#[derive(Debug)]
pub struct MarcRecord<'s> {
    header: MarcHeader<'s>,
    data: &'s [u8],
}

pub struct MarcRecordEntries<'s> {
    directory: MarcDirectory<'s>,
    record_payload: &'s [u8],
}

#[derive(std::cmp::PartialEq)]
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

// todo we want to iter over this
pub struct MarcDirectory<'s> {
    directory: &'s [u8],
}

#[derive(Debug)]
pub struct MarcDirectoryEntryRef<'s> {
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
    pub fn get_entry(&self, i: usize) -> MarcDirectoryEntryRef {
        MarcDirectoryEntryRef {
            entry: &self.directory[12 * i..12 * (i + 1)],
        }
    }
    pub fn num_entries(&self) -> usize {
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
        self.data.len() + 24
    }
    pub fn directory(&self) -> MarcDirectory<'s> {
        let directory_end = end_of_entry_position(&self.data);
        MarcDirectory {
            directory: &self.data[0..directory_end.expect("malformed entry")],
        }
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
#[derive(Debug)]
pub struct MarcRecordBatch<'s> {
    pub records: Vec<MarcRecord<'s>>,
}

#[derive(Debug)]
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
        dbg!(read);
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

#[cfg(test)]
mod tests {
    use crate::MarcReader;
    use std::io::BufReader;
    use std::io::Cursor;
    use std::io::Seek;
    use std::io::SeekFrom;
    #[test]
    fn read_one() -> Result<(), String> {
        let str = "00827nz  a2200241nc 4500\
001001000000\
003000700010\
005001700017\
008004100034\
024005100075\
035002200126\
035002200148\
035002900170\
040004000199\
042000900239\
065001600248\
075001400264\
079000900278\
083004200287\
150001200329\
550019200341\
670001200533\
913004000545\
040000028DE-10120100106125650.0880701n||azznnbabn           | ana    |c7 a4000002-30http://d-nb.info/gnd/4000002-32gnd  a(DE-101)040000028  a(DE-588)4000002-3  z(DE-588c)4000002-39v:zg  aDE-101cDE-1019r:DE-101bgerd0832  agnd1  a31.9b2sswd  bs2gndgen  agqs04a621.3815379d:29t:2010-01-06223/ger  aA 302 D  0(DE-101)0402724270(DE-588)4027242-40https://d-nb.info/gnd/4027242-4aIntegrierte Schaltung4obal4https://d-nb.info/standards/elementset/gnd#broaderTermGeneralwriOberbegriff allgemein  aVorlage  SswdisaA 302 D0(DE-588c)4000002-3".as_bytes();
        dbg!(str.len());
        let c = Cursor::new(str);
        let mut breader = BufReader::new(c);
        let mut mreader = MarcReader::new(breader);
        let mut v: Vec<u8> = Vec::new();
        v.resize(10000, 0);
        let r = mreader.read_batch(&mut v);
        match r {
            Ok(Some(batch)) => {
                assert_eq!(batch.records.len(), 1);
                let record = &batch.records[0];
                assert_eq!(record.record_length(), 827);
                let dir = record.directory();
                dbg!(std::str::from_utf8(dir.directory));
                assert_eq!(dir.num_entries(), 18);
                let entry_types = [
                    1, 3, 5, 8, 24, 35, 35, 35, 40, 42, 65, 75, 79, 83, 150, 550, 670, 913,
                ];
                let entry_lengths = [
                    10, 7, 17, 41, 51, 22, 22, 29, 40, 9, 16, 14, 9, 42, 12, 192, 12, 40,
                ];
                let entry_starts = [
                    0, 10, 17, 34, 75, 126, 148, 170, 199, 239, 248, 264, 278, 287, 329, 341, 533,
                    545,
                ];

                for i in 0..18 {
                    let entry = dir.get_entry(i);
                    dbg!(std::str::from_utf8(entry.entry));
                    assert_eq!(entry.entry_type(), entry_types[i], "i {}", i);
                    assert_eq!(entry.len(), entry_lengths[i], "i {}", i);
                    assert_eq!(entry.start(), entry_starts[i], "i {}", i);
                }
                Ok(())
            }
            _ => Err("something bad".to_string()),
        }
    }
}
