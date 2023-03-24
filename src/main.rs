#![allow(dead_code)]
use std::env;
use std::fs::File;
use std::io::{Read, Seek};
pub mod compiler;
pub mod exprparse;
pub mod field_expression;
pub mod filter;
pub mod lexer;
pub mod marcrecord;
pub mod ownedrecord;
pub mod parsedrecord;
pub mod parser;
pub mod projection;
pub mod record;
pub mod util;

//use filter::*;
use marcrecord::MarcHeader;
use marcrecord::MarcReader;
//use marcrecord::MarcRecord;
//use parsedrecord::*;
use record::*;

fn get_header(data: &[u8]) -> MarcHeader {
    MarcHeader {
        header: &data[0..24],
    }
}

pub fn print_record(r: &dyn Record) {
    print!("Record: ");
    for field in r.field_iter(None) {
        println!("{}\t{}", field.field_type, field.utf8_data());
    }
}

fn find_table(table_name: &str) -> Result<MarcReader<File>, std::io::Error> {
    let reader = File::open(format!("{}.mrc", table_name))?;
    Ok(MarcReader::new(reader))
}

pub fn run_sql<T, H>(
    sql_text: &str,
    make_reader: fn(&str) -> Result<MarcReader<T>, std::io::Error>,
    mut handle_record: H,
) -> Result<(), String>
where
    T: Seek + Read,
    H: FnMut(&dyn Record) -> (),
{
    let mut compile_result = compiler::compile(sql_text)?;
    let mut marc_reader = make_reader(&compile_result.table_name).unwrap();
    let projection = compile_result.projection;
    let filter = compile_result.filter_expr;

    let cap = 128 * 1024 * 1024;
    let mut mem: Vec<u8> = vec![0; cap];

    while let Ok(Some(batch)) = marc_reader.read_batch(mem.as_mut_slice()) {
        let mut boxs: Vec<Box<dyn Record>> = batch
            .records
            .into_iter()
            .map(|x| -> Box<dyn Record> { Box::new(x) })
            .collect();
        let remaining = filter
            .as_ref()
            .map(|x| x.filter(&mut boxs).0)
            .unwrap_or(boxs.len());
        projection.project(&mut boxs[..remaining]);
        for r in boxs.into_iter().take(remaining) {
            //for r in batch.records {
            handle_record(&*r);
            //r.to_marc21(&mut stdout);
            //stdout.write(b"\n");
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    run_sql(&args[1], find_table, |x: &dyn Record| {
        print_record(x);
    })?;
    Ok(())
    //    let filename = &args[1];
    //    let filter_str = &args[2];
    //    //let reader = BufReader::new(File::open(filename).unwrap());
    //    let reader = File::open(filename).unwrap();
    //    let mut marc_reader = MarcReader::new(reader);
    //    let cap = 128 * 1024 * 1024;
    //    let mut mem: Vec<u8> = vec![0; cap];
    //    let filter = compiler::compile(filter_str)?.filter_expr.unwrap();
    //    let mut i = 0_usize;
    //
    //    let select_expr: Box<dyn FieldExpression> = Box::new(FieldTypeSelect::new(vec![1, 150, 400]));
    //    let proj = Projection::new(vec![select_expr]);
    //    while let Ok(Some(batch)) = marc_reader.read_batch(mem.as_mut_slice()) {
    //        i += batch.records.len();
    //        //				for record in batch.records {
    //        //					for field in record.field_iter(Some(1)) {
    //        //						if field.data == "1264401221".as_bytes() {
    //        //							print_record(&record);
    //        //							println!();
    //        //							break;
    //        //						}
    //        //					}
    //        //				}
    //
    //        let mut boxs: Vec<Box<dyn Record>> = batch
    //            .records
    //            .into_iter()
    //            .map(|x| -> Box<dyn Record> { Box::new(x) })
    //            .collect();
    //        let remaining = filter.filter(&mut boxs);
    //        proj.project(&mut boxs[..remaining]);
    //        for r in boxs.into_iter().take(remaining) {
    //            //for r in batch.records {
    //            print_record(&*r);
    //            println!("{}", i);
    //            //r.to_marc21(&mut stdout);
    //            //stdout.write(b"\n");
    //        }
    //    }
    //    println!("{}", i);
    //    Ok(())
    //    /*
    //        let args: Vec<String> = env::args().collect();
    //        let filename = &args[1];
    //        let field_type = args[2].parse::<usize>().unwrap();
    //        let regex = &args[3];
    //        let mut reader = BufReader::new(File::open(filename).unwrap());
    //        let mut marc_reader = MarcReader::new(reader);
    //        let cap = 64 * 1024 * 1024;
    //        let mut mem: Vec<u8> = Vec::with_capacity(cap);
    //        mem.resize(cap, 0);
    //        let mut num_records: usize = 0;
    //        while let Ok(Some(mut batch)) = marc_reader.read_batch(mem.as_mut_slice()) {
    //            num_records += batch.records.len();
    //            //let mut parsed_records: Vec<ParsedRecord> =
    //            //   batch.records.iter().map(|r| ParsedRecord::new(r)).collect();
    //            //for r in parsed_records.iter() {
    //            //  for f in r.field_iter(Some(150)) {
    //            //    dbg!(f.utf8_data());
    //            //  }
    //            //}
    //            //RegexFilter::new(Some(field_type), regex).filter(&mut parsed_records);
    //            //for r in parsed_records {
    //            //    print_record(r);
    //            //    println!();
    //            //}
    //            //RegexFilter::new(Some(field_type), regex).filter(&mut batch.records);
    //            let mut boxs: Vec<Box<dyn Record>> = batch
    //                .records
    //                .into_iter()
    //                .map(|x| -> Box<dyn Record> { Box::new(x) })
    //                .collect();
    //            let regexFilter = Box::new(RegexFilter::new(Some(field_type), regex));
    //            let regexFilter2 = Box::new(RegexFilter::new(Some(150), "Katze"));
    //            let i = OrFilter::new(vec![regexFilter, regexFilter2]).filter(&mut boxs);
    //            //let i = RegexFilter::new(Some(field_type), regex).filter(&mut boxs);
    //            let mut stdout = std::io::stdout();
    //            use std::io::Write;
    //            for r in boxs.into_iter().take(i) {
    //                //for r in batch.records {
    //                print_record(&*r);
    //                println!();
    //                //r.to_marc21(&mut stdout);
    //                //stdout.write(b"\n");
    //            }
    //            //      for r in batch.records {
    //            //        let l = r.header().record_length();
    //            //        assert!(r.header().record_length() == r.record_length());
    //            //        //dbg!(str::from_utf8(&r.data));
    //            //        if l < 10 {
    //            //            println!("0000{}", l);
    //            //        } else if l < 100 {
    //            //            println!("000{}", l);
    //            //        } else if l < 1000 {
    //            //            println!("00{}", l);
    //            //        } else if l < 10000 {
    //            //            println!("0{}", l);
    //            //        } else {
    //            //            println!("{}", l);
    //            //        }
    //            //      }
    //        }
    //        //    let contents = fs::read(filename).expect("can't read");
    //        //
    //        //    dbg!(contents.len());
    //        //    let mut offset = 0;
    //        //    while offset < contents.len() {
    //        //        let h = get_header(&contents[offset..]);
    //        //        let h_len = h.record_length();
    //        //        let r = MarcRecord::new(h, &contents[offset..offset + h_len]);
    //        //        offset += h_len;
    //        //        let l = r.header().record_length();
    //        //        assert!(r.header().record_length() == r.record_length());
    //        //        //dbg!(str::from_utf8(&r.data));
    //        //        if l < 10 {
    //        //            println!("0000{}", l);
    //        //        } else if l < 100 {
    //        //            println!("000{}", l);
    //        //        } else if l < 1000 {
    //        //            println!("00{}", l);
    //        //        } else if l < 10000 {
    //        //            println!("0{}", l);
    //        //        } else {
    //        //            println!("{}", l);
    //        //        }
    //        //        //      let d = r.directory();
    //        //        //      dbg!(str::from_utf8(d.directory));
    //        //        //      for i in 0..d.num_entries() {
    //        //        //        let d_e = d.get_entry(i);
    //        //        //        dbg!(&d_e);
    //        //        //      }
    //        //    }
    //    */
}

#[cfg(test)]
mod test {
    use crate::marcrecord::MarcReader;
    use crate::ownedrecord::*;
    use crate::record::*;
    use crate::{print_record, run_sql};
    use std::io::BufReader;
    use std::io::Cursor;

    static STR : &[u8]= "00827nz  a2200241nc 4500\
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

    fn test_reader(
        _: &str,
    ) -> Result<MarcReader<BufReader<Cursor<&'static [u8]>>>, std::io::Error> {
        let c = Cursor::new(STR);
        let breader = BufReader::new(c);
        Ok(MarcReader::new(breader))
    }

    #[test]
    fn test1() -> Result<(), String> {
        let mut v: Vec<OwnedRecord> = Vec::new();
        run_sql("select * from bla", test_reader, |r: &dyn Record| {
            let mut or = OwnedRecord::new();
            or.add_field_from_iter(&mut r.field_iter(None));
            v.push(or);
        })?;
        assert_eq!(v.len(), 1);
        Ok(())
    }
}
