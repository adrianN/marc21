use crate::marcrecord::*;

pub enum AuthorityRecordStatus {
    IncreaseEncodingLevel = b'a' as isize,
    Corrected = b'c' as isize,
    Deleted = b'd' as isize,
    New = b'n' as isize,
    Obsolete = b'o' as isize,
    Split = b's' as isize,
    Replaced = b'x' as isize,
}

pub enum AuthorityRecordCharacterCodingScheme {
    Marc8 = b'#' as isize,
    Unicode = b'a' as isize,
}

pub struct AuthorityRecordMeta {
    record_type: RecordType,
    status: AuthorityRecordStatus,
    character_coding_scheme: AuthorityRecordCharacterCodingScheme,
    // TODO we probably want to use an arena for these
    field_types: Vec<u16>,
    field_offsets: Vec<usize>,
    field_lengths: Vec<usize>,
}

// TODO we could implement a builder pattern to reuse things we already
// parsed during pre-filtering
impl AuthorityRecordMeta {
    pub fn new(r: &MarcRecord) -> AuthorityRecordMeta {
        let t = r.header().record_type();

        // todo check whether other record types than authority parse differently here
        // and maybe move stuff to MarcHeader
        let s = match r.header().header[5] {
            b'a' => AuthorityRecordStatus::IncreaseEncodingLevel,
            b'c' => AuthorityRecordStatus::Corrected,
            b'c' => AuthorityRecordStatus::Deleted,
            b'n' => AuthorityRecordStatus::New,
            b'o' => AuthorityRecordStatus::Obsolete,
            b's' => AuthorityRecordStatus::Split,
            b'x' => AuthorityRecordStatus::Replaced,
            _ => panic!("oopsie"),
        };

        let coding_scheme = match r.header().header[9] {
            b'a' => AuthorityRecordCharacterCodingScheme::Unicode,
            _ => unimplemented!(),
        };

        // todo the remaining fields of the header

        let dir_len = r.directory();

        let mut field_types = Vec::new();
        let mut field_offsets = Vec::new();
        let mut field_lengths = Vec::new();

        AuthorityRecordMeta {
            record_type: t,
            status: s,
            character_coding_scheme: coding_scheme,
            field_types: field_types,
            field_offsets: field_offsets,
            field_lengths: field_lengths,
        }
    }
}

pub enum RecordMeta {
    AuthorityMeta(AuthorityRecordMeta),
}

impl RecordMeta {
    pub fn new(r: &MarcRecord) -> RecordMeta {
        match r.header().record_type() {
            RecordType::Authority => RecordMeta::AuthorityMeta(AuthorityRecordMeta::new(r)),
            _ => todo!(),
        }
    }
}

pub struct Record {
    meta: RecordMeta,
    // Todo we definitely want to use an arena for this
    data: Vec<u8>,
}

impl Record {
    pub fn new(r: &MarcRecord) -> Record {
        Record {
            meta: RecordMeta::new(r),
            data: r.data().to_vec(),
        }
    }
}
