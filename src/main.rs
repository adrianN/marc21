use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
pub mod marcrecord;
pub mod util;

use marcrecord::MarcHeader;
use marcrecord::MarcRecord;

fn get_header(data: &[u8]) -> MarcHeader {
    MarcHeader {
        header: &data[0..24],
    }
}

struct MarcRecordBatch<'s> {
    records: Vec<MarcRecord<'s>>,
}

struct MarcReader<R>
where
    R: Read + Seek,
{
    base_reader: BufReader<R>,
}

impl<R> MarcReader<R>
where
    R: Read + Seek,
{
    fn read_batch<'s>(&mut self, mem: &'s mut [u8]) -> Result<Option<MarcRecordBatch<'s>>, std::io::Error> {
        let mut records: Vec<MarcRecord> = Vec::new();
        let mut i = 0;
        let capacity = mem.len();
        let start_pos = self.base_reader.stream_position().unwrap();
        let read = self.base_reader.read(mem)?;
        if read == 0 { return Ok(None); }
        while i + 24 < read {
            let header = MarcHeader {
                header: &mem[i..i + 24],
            };
            let record_length = header.record_length();
            assert!(record_length < mem.len());
            if record_length + i <= read {
                // still fits in mem
                records.push(MarcRecord::new(
                    header,
                    &mem[i + 24..i + record_length],
                ));
                i += record_length;
            } else {
                break;
            }
        }
        // mem full, backpedal
        //self.base_reader.seek_relative(-24);
        // TODO seek_relative is unstable in my version of rust
        self.base_reader.seek(SeekFrom::Start(start_pos + i as u64));
        let num_bytes = records.iter().map(|r| r.header().record_length()).sum::<usize>() as u64;
        let stream_pos = self.base_reader.stream_position().unwrap() ;
        let bytes_consumed = stream_pos - start_pos ;
        assert!(bytes_consumed== (num_bytes));

        Ok(Some(MarcRecordBatch { records: records }))
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut reader = BufReader::new(File::open(filename).unwrap());
    let mut marc_reader= MarcReader { base_reader : reader };
    let cap = 64*1024;//*1024;
    let mut mem : Vec<u8> = Vec::with_capacity(cap);
    mem.resize(cap, 0);
    let mut num_records : usize = 0;
    while let Ok(Some(batch)) = marc_reader.read_batch(mem.as_mut_slice()) {
      num_records += batch.records.len();
//      for r in batch.records {
//        let l = r.header().record_length();
//        assert!(r.header().record_length() == r.record_length());
//        //dbg!(str::from_utf8(&r.data));
//        if l < 10 {
//            println!("0000{}", l);
//        } else if l < 100 {
//            println!("000{}", l);
//        } else if l < 1000 {
//            println!("00{}", l);
//        } else if l < 10000 {
//            println!("0{}", l);
//        } else {
//            println!("{}", l);
//        }
//      }
    }
    dbg!(num_records);
//    let contents = fs::read(filename).expect("can't read");
//
//    dbg!(contents.len());
//    let mut offset = 0;
//    while offset < contents.len() {
//        let h = get_header(&contents[offset..]);
//        let h_len = h.record_length();
//        let r = MarcRecord::new(h, &contents[offset..offset + h_len]);
//        offset += h_len;
//        let l = r.header().record_length();
//        assert!(r.header().record_length() == r.record_length());
//        //dbg!(str::from_utf8(&r.data));
//        if l < 10 {
//            println!("0000{}", l);
//        } else if l < 100 {
//            println!("000{}", l);
//        } else if l < 1000 {
//            println!("00{}", l);
//        } else if l < 10000 {
//            println!("0{}", l);
//        } else {
//            println!("{}", l);
//        }
//        //      let d = r.directory();
//        //      dbg!(str::from_utf8(d.directory));
//        //      for i in 0..d.num_entries() {
//        //        let d_e = d.get_entry(i);
//        //        dbg!(&d_e);
//        //      }
//    }
}
