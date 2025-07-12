use binrw::binrw;

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooRamEntries {
    pub entry_count: u32,
    #[br(count = entry_count)]
    pub entries: Vec<MooRamEntry>,
}

#[derive(Debug, Default)]
#[binrw]
#[brw(little)]
pub struct MooRamEntry {
    pub address: u32,
    pub value: u8,
}
