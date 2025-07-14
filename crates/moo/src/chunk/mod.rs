use crate::types::MooCpuType;
use binrw::{binrw, BinResult, BinWrite};
use std::io::{Cursor, Seek, Write};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug)]
#[binrw]
#[brw(little)]
pub enum MooChunkType {
    #[brw(magic = b"MOO ")]
    FileHeader,
    #[brw(magic = b"TEST")]
    TestHeader,
    #[brw(magic = b"NAME")]
    Name,
    #[brw(magic = b"BYTS")]
    Bytes,
    #[brw(magic = b"INIT")]
    InitialState,
    #[brw(magic = b"FINA")]
    FinalState,
    #[brw(magic = b"REGS")]
    Registers16,
    #[brw(magic = b"RGS2")]
    XRegisters,
    #[brw(magic = b"RAM ")]
    Ram,
    #[brw(magic = b"QUEU")]
    QueueState,
    #[brw(magic = b"CYCL")]
    CycleStates,
    #[brw(magic = b"HASH")]
    Hash,
}

impl MooChunkType {
    pub fn write<WS, T>(&self, writer: &mut WS, payload: &T) -> BinResult<()>
    where
        WS: Write + Seek,
        T: BinWrite + binrw::meta::WriteEndian,
        for<'a> <T as BinWrite>::Args<'a>: Default,
    {
        let mut payload_buf = Cursor::new(Vec::new());

        payload.write_le(&mut payload_buf)?;

        let chunk = MooChunkHeader {
            chunk_type: *self,
            size: payload_buf.position() as u32,
        };

        // Write the chunk header
        chunk.write_le(writer)?;
        // Write the data
        writer
            .write_all(&payload_buf.into_inner())
            .map_err(|e| binrw::Error::Io(e))
    }
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooChunkHeader {
    pub chunk_type: MooChunkType,
    pub size: u32,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooFileHeader {
    pub version: u8,
    pub reserved: [u8; 3],
    pub test_count: u32,
    pub cpu_name: [u8; 4],
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooTestChunk {
    pub index: u32,
}

#[binrw]
#[brw(little)]
pub struct MooNameChunk {
    pub len: u32,
    #[br(count = len)]
    #[br(map = |x: Vec<u8>| String::from_utf8_lossy(&x).to_string())]
    #[bw(map = |x: &String| x.as_bytes().to_vec())]
    pub name: String,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooBytesChunk {
    pub len: u32,
    #[br(count = len)]
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooHashChunk {
    pub hash: [u8; 20],
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
}

#[derive(Debug, Default)]
#[binrw]
#[brw(little)]
pub struct MooFileMetadata {
    pub version: u8,
    pub cpu_type: MooCpuType,
    pub opcode: u32,
    pub test_ct: u32,
    pub file_seed: u32,
    pub flag_mask: u32,
    pub mnemonic_len: u8,
    #[br(count = mnemonic_len)]
    pub mnemonic: Vec<u8>,
    pub filename_len: u8,
    #[br(count = filename_len)]
    pub filename: Vec<u8>,
}

impl MooFileMetadata {
    pub fn new(version: u8, cpu_type: MooCpuType, opcode: u32) -> Self {
        Self {
            version,
            cpu_type,
            opcode,
            ..Default::default()
        }
    }

    pub fn with_test_count(mut self, test_count: u32) -> Self {
        self.test_ct = test_count;
        self
    }
    pub fn with_file_seed(mut self, file_seed: u32) -> Self {
        self.file_seed = file_seed;
        self
    }
    pub fn with_flag_mask(mut self, flag_mask: u32) -> Self {
        self.flag_mask = flag_mask;
        self
    }
    pub fn with_mnemonic(mut self, mnemonic: String) -> Self {
        let mnemonic = mnemonic.into_bytes();
        self.mnemonic_len = mnemonic.len() as u8;
        self.mnemonic = mnemonic;
        self
    }
    pub fn with_filename(mut self, filename: PathBuf) -> Self {
        let filename = filename
            .into_os_string()
            .into_string()
            .unwrap_or_default()
            .into_bytes();
        self.filename_len = filename.len() as u8;
        self.filename = filename;
        self
    }
    pub fn mnemonic(&self) -> String {
        String::from_utf8_lossy(&self.mnemonic).to_string()
    }
    pub fn filename(&self) -> PathBuf {
        PathBuf::from(String::from_utf8_lossy(&self.filename).to_string())
    }
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooTestGenMetadata {
    pub seed: u32,
    pub gen_ct: u16,
    pub error_ct: u16,
    pub shutdown_ct: u16,
}
