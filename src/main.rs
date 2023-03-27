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
    H: FnMut(&dyn Record),
{
    let mut compile_result = compiler::compile(sql_text)?;
    let mut marc_reader = make_reader(&compile_result.table_name).unwrap();
    let projection = compile_result.projection;
    let filter = compile_result.filter_expr;

    let cap = 128 * 1024 * 1024;
    let mut mem: Vec<u8> = vec![0; cap];

    while let Some(batch) = marc_reader
        .read_batch(mem.as_mut_slice())
        .map_err(|x| format!("{}", x))?
    {
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
040000028DE-10120100106125650.0880701n||azznnbabn           | ana    |c7 a4000002-30http://d-nb.info/gnd/4000002-32gnd  a(DE-101)040000028  a(DE-588)4000002-3  z(DE-588c)4000002-39v:zg  aDE-101cDE-1019r:DE-101bgerd0832  agnd1  a31.9b2sswd  bs2gndgen  agqs04a621.3815379d:29t:2010-01-06223/ger  aA 302 D  0(DE-101)0402724270(DE-588)4027242-40https://d-nb.info/gnd/4027242-4aIntegrierte Schaltung4obal4https://d-nb.info/standards/elementset/gnd#broaderTermGeneralwriOberbegriff allgemein  aVorlage  SswdisaA 302 D0(DE-588c)4000002-3\
02554naa a2200553uc 4500\
001001100000\
003000700011\
005001700018\
007001500035\
008004100050\
016002300091\
022001400114\
024003500128\
024004900163\
035002600212\
035002200238\
040002800260\
041000800288\
044001000296\
082003300306\
083002900339\
084001400368\
100003600382\
245023000418\
300003900648\
336002600687\
337003200713\
338003700745\
506009200782\
583007000874\
653002000944\
653002600964\
653003700990\
700002801027\
700003301055\
700003101088\
700003301119\
700003501152\
700002801187\
710004901215\
773008401264\
773019001348\
850002101538\
856006401559\
856008801623\
856007701711\
856004601788\
883008301834\
883008301917\
1203058578DE-10120200120180536.0cr||||||||||||200118s2019    gw |||||o|||| 00||||eng  7 2DE-101a1203058578  a1479-58767 a10.1186/s12967-019-2032-y2doi7 2urnaurn:nbn:de:101:1-2020011823361862943632  a(DE-599)DNB1203058578  a(OCoLC)1196655458  a1140bgercDE-101d9999  aeng  cXA-DE7481\\pa616.994qDE-101223kdnb7 82\\pa610qDE-101223sdnb  aR-RZ2lcc1 aZeng, Jiang-huieVerfasser4aut10aPrognosis of clear cell renal cell carcinoma (ccRCC) based on a six-lncRNA-based risk score: an investigation based on RNA-sequencing datacby Jiang-hui Zeng, Wei Lu, Liang Liang, Gang Chen, Hui-hua Lan, Xiu-Yun Liang, Xu Zhu  aOnline-Ressourcebonline resource.  aTextbtxt2rdacontent  aComputermedienbc2rdamedia  aOnline-Ressourcebcr2rdacarrier0 aOpen AccessfUnrestricted online accessuhttp://purl.org/coar/access_right/c_abf22star1 aArchivierung/Langzeitarchivierung gewährleistet5DE-1012pdager 0a(lcsh)Medicine. 0aBiomedicine, general. 0aMedicine/Public Health, general.1 aLu, WeieVerfasser4aut1 aLiang, LiangeVerfasser4aut1 aChen, GangeVerfasser4aut1 aLan, Hui-huaeVerfasser4aut1 aLiang, Xiu-YuneVerfasser4aut1 aZhu, XueVerfasser4aut2 aSpringerLink (Online service)eSonstige4oth187|||sgvolume:17gnumber:1gday:23gmonth:8gyear:2019gpages:1-20gdate:12.201908iEnthalten intJournal of translational medicinedLondon : BioMed Central, 2003-hOnline-Ressourceg17, Heft 1 (23.8.2019), 1-20, 12.2019w(DE-600)2118570-0w(DE-101)02505497Xx1479-5876  aDE-101aaDE-101b40uhttps://doi.org/10.1186/s12967-019-2032-yxResolving-System40uhttps://nbn-resolving.org/urn:nbn:de:101:1-2020011823361862943632xResolving-System 0uhttps://d-nb.info/1203058578/34xLangzeitarchivierung Nationalbibliothek4 uhttps://doi.org/10.1186/s12967-019-2032-y0 81\\paaepknc0,98426d20200119qDE-101uhttps://d-nb.info/provenance/plan#aepkn0 82\\paaepsgc0,99929d20200119qDE-101uhttps://d-nb.info/provenance/plan#aepsg".as_bytes();

    fn test_reader(
        _: &str,
    ) -> Result<MarcReader<BufReader<Cursor<&'static [u8]>>>, std::io::Error> {
        let c = Cursor::new(STR);
        let breader = BufReader::new(c);
        Ok(MarcReader::new(breader))
    }

    #[test]
    fn test_select_star() -> Result<(), String> {
        let mut v: Vec<OwnedRecord> = Vec::new();
        run_sql("select * from bla", test_reader, |r: &dyn Record| {
            let mut or = OwnedRecord::new();
            or.add_field_from_iter(&mut r.field_iter(None));
            v.push(or);
        })?;
        assert_eq!(v.len(), 2);
        let num_fields: Vec<usize> = v.iter().map(|x| x.field_iter(None).count()).collect();
        assert_eq!(num_fields, vec![18, 44]);
        Ok(())
    }

    #[test]
    fn test_select() -> Result<(), String> {
        let mut v: Vec<OwnedRecord> = Vec::new();
        run_sql("select 700, 42 from bla", test_reader, |r: &dyn Record| {
            let mut or = OwnedRecord::new();
            or.add_field_from_iter(&mut r.field_iter(None));
            v.push(or);
        })?;
        assert_eq!(v.len(), 2);
        let num_fields: Vec<usize> = v.iter().map(|x| x.field_iter(None).count()).collect();
        assert_eq!(num_fields, vec![1, 6]);
        Ok(())
    }

    #[test]
    fn test_select_2() -> Result<(), String> {
        let mut v: Vec<OwnedRecord> = Vec::new();
        run_sql("select 9999 from bla", test_reader, |r: &dyn Record| {
            let mut or = OwnedRecord::new();
            or.add_field_from_iter(&mut r.field_iter(None));
            v.push(or);
        })?;
        assert_eq!(v.len(), 2);
        let num_fields: Vec<usize> = v.iter().map(|x| x.field_iter(None).count()).collect();
        assert_eq!(num_fields, vec![0, 0]);
        Ok(())
    }

    #[test]
    fn test_not_null_field_ref() -> Result<(), String> {
        let mut v: Vec<OwnedRecord> = Vec::new();
        run_sql(
            "select * from bla where not_null(42)",
            test_reader,
            |r: &dyn Record| {
                let mut or = OwnedRecord::new();
                or.add_field_from_iter(&mut r.field_iter(None));
                v.push(or);
            },
        )?;
        assert_eq!(v.len(), 1);
        let num_fields: Vec<usize> = v.iter().map(|x| x.field_iter(None).count()).collect();
        assert_eq!(num_fields, vec![18]);
        Ok(())
    }

    #[test]
    fn test_not_null_expr() -> Result<(), String> {
        let mut v: Vec<OwnedRecord> = Vec::new();
        run_sql(
            "select * from bla where not_null(42 ~ 'aoe')",
            test_reader,
            |r: &dyn Record| {
                let mut or = OwnedRecord::new();
                or.add_field_from_iter(&mut r.field_iter(None));
                v.push(or);
            },
        )?;
        assert_eq!(v.len(), 1);
        let num_fields: Vec<usize> = v.iter().map(|x| x.field_iter(None).count()).collect();
        assert_eq!(num_fields, vec![18]);
        Ok(())
    }

    #[test]
    fn test_is_null_field_ref() -> Result<(), String> {
        let mut v: Vec<OwnedRecord> = Vec::new();
        run_sql(
            "select * from bla where is_null(42)",
            test_reader,
            |r: &dyn Record| {
                let mut or = OwnedRecord::new();
                or.add_field_from_iter(&mut r.field_iter(None));
                v.push(or);
            },
        )?;
        assert_eq!(v.len(), 1);
        let num_fields: Vec<usize> = v.iter().map(|x| x.field_iter(None).count()).collect();
        assert_eq!(num_fields, vec![44]);
        Ok(())
    }

    #[test]
    fn test_is_null_expr() -> Result<(), String> {
        let mut v: Vec<OwnedRecord> = Vec::new();
        run_sql(
            "select * from bla where is_null(42 ~ 'aou')",
            test_reader,
            |r: &dyn Record| {
                let mut or = OwnedRecord::new();
                or.add_field_from_iter(&mut r.field_iter(None));
                v.push(or);
            },
        )?;
        assert_eq!(v.len(), 1);
        let num_fields: Vec<usize> = v.iter().map(|x| x.field_iter(None).count()).collect();
        assert_eq!(num_fields, vec![44]);
        Ok(())
    }

    #[test]
    fn test_not() -> Result<(), String> {
        let mut v: Vec<OwnedRecord> = Vec::new();
        run_sql(
            "select * from bla where not(is_null(42))",
            test_reader,
            |r: &dyn Record| {
                let mut or = OwnedRecord::new();
                or.add_field_from_iter(&mut r.field_iter(None));
                v.push(or);
            },
        )?;
        assert_eq!(v.len(), 1);
        let num_fields: Vec<usize> = v.iter().map(|x| x.field_iter(None).count()).collect();
        assert_eq!(num_fields, vec![18]);
        Ok(())
    }
}
