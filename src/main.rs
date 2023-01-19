use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
pub mod filter;
pub mod marcrecord;
pub mod parsedrecord;
pub mod util;

use marcrecord::MarcHeader;
use marcrecord::MarcReader;
use marcrecord::MarcRecord;
use parsedrecord::*;

fn get_header(data: &[u8]) -> MarcHeader {
    MarcHeader {
        header: &data[0..24],
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut reader = BufReader::new(File::open(filename).unwrap());
    let mut marc_reader = MarcReader::new(reader);
    let cap = 64 * 1024 * 1024;
    let mut mem: Vec<u8> = Vec::with_capacity(cap);
    mem.resize(cap, 0);
    let mut num_records: usize = 0;
    while let Ok(Some(batch)) = marc_reader.read_batch(mem.as_mut_slice()) {
        num_records += batch.records.len();
        for r in batch.records {
            let _parsed_record = Record::new(&r);
        }
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
