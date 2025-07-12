use crate::chunk::{MooBytesChunk, MooChunkType, MooFileHeader, MooNameChunk, MooTestChunk};
use crate::types::{MooCycleState, MooRamEntries, MooRamEntry};
use crate::types::{MooRegisters1, MooRegisters1Init};
use binrw::{BinResult, BinWrite};
use sha1::Digest;
use std::io::{Cursor, Write};

pub struct MooTestState {
    pub regs: MooRegisters1,
    pub queue: Vec<u8>,
    pub ram: MooRamEntries,
}

impl MooTestState {
    pub fn new(
        regs_start: &MooRegisters1Init,
        regs_final: Option<&MooRegisters1Init>,
        queue: Vec<u8>,
        ram: Vec<MooRamEntry>,
    ) -> Self {
        let regs = if let Some(final_regs) = regs_final {
            MooRegisters1::from((regs_start, final_regs))
        } else {
            MooRegisters1::from(regs_start)
        };

        let ram_entries = MooRamEntries {
            entry_count: ram.len() as u32,
            entries: ram,
        };
        Self {
            regs,
            queue,
            ram: ram_entries,
        }
    }

    pub fn regs(&self) -> &MooRegisters1 {
        &self.regs
    }

    pub fn queue(&self) -> &[u8] {
        &self.queue
    }

    pub fn ram(&self) -> &[MooRamEntry] {
        &self.ram.entries
    }

    pub fn write<W: Write + std::io::Seek>(&self, _writer: &mut W) -> BinResult<()> {
        // // Write the registers
        // self.regs.write_le(writer)?;
        //
        // // Write the queue
        // writer.write_all(&self.queue)?;
        //
        // // Write the RAM entries
        //
        // for entry in &self.ram.entries {
        //     writer.write_all(&entry.address.to_le_bytes())?;
        //     writer.write_all(&entry.value.to_le_bytes())?;
        // }

        Ok(())
    }
}

pub struct MooTest {
    name: String,
    bytes: Vec<u8>,
    initial_state: MooTestState,
    final_state: MooTestState,
    cycles: Vec<MooCycleState>,
}

impl MooTest {
    pub fn new(
        name: String,
        bytes: &[u8],
        initial_state: MooTestState,
        final_state: MooTestState,
        cycles: &[MooCycleState],
    ) -> Self {
        Self {
            name,
            bytes: bytes.to_vec(),
            initial_state,
            final_state,
            cycles: cycles.to_vec(),
        }
    }
}

pub struct MooTestFile {
    version: u8,
    arch: String,
    tests: Vec<MooTest>,
}

impl MooTestFile {
    pub fn new(version: u8, arch: String, capacity: usize) -> Self {
        Self {
            version,
            arch,
            tests: Vec::with_capacity(capacity),
        }
    }

    pub fn add_test(&mut self, test: MooTest) {
        self.tests.push(test);
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn arch(&self) -> &str {
        &self.arch
    }

    pub fn tests(&self) -> &[MooTest] {
        &self.tests
    }

    pub fn write<W: Write + std::io::Seek>(&self, writer: &mut W) -> BinResult<()> {
        // Write the file header chunk.

        MooChunkType::FileHeader.write(
            writer,
            &MooFileHeader {
                version: self.version,
                reserved: [0; 3],
                test_count: self.tests.len() as u32,
                cpu_name: self.arch.clone().into_bytes()[0..4]
                    .try_into()
                    .expect("CPU Name must be 4 chars long"),
            },
        )?;

        for (ti, test) in self.tests.iter().enumerate() {
            let mut test_buffer = Cursor::new(Vec::new());

            // Write the test chunk body.
            MooTestChunk { index: ti as u32 }.write(&mut test_buffer)?;

            // Write the name chunk.
            let mut name_buffer = Cursor::new(Vec::new());
            MooNameChunk {
                len: test.name.len() as u32,
            }
            .write(&mut name_buffer)?;
            name_buffer.write_all(test.name.as_bytes())?;
            MooChunkType::Name.write(&mut test_buffer, &name_buffer.into_inner())?;

            // Write the bytes chunk.
            let mut bytes_buffer = Cursor::new(Vec::new());
            MooBytesChunk {
                len: test.bytes.len() as u32,
            }
            .write(&mut bytes_buffer)?;
            bytes_buffer.write_all(test.bytes.as_slice())?;
            MooChunkType::Bytes.write(&mut test_buffer, &bytes_buffer.into_inner())?;

            let mut initial_state_buffer = Cursor::new(Vec::new());

            // Write the initial regs.
            MooChunkType::Registers16.write(&mut initial_state_buffer, &test.initial_state.regs)?;

            // Write the initial queue, if not empty.
            if !test.initial_state.queue.is_empty() {
                MooChunkType::QueueState
                    .write(&mut initial_state_buffer, &test.initial_state.queue)?;
            }

            // Write the initial ram.
            MooChunkType::Ram.write(&mut initial_state_buffer, &test.initial_state.ram)?;

            // Write the initial state chunk.
            MooChunkType::InitialState
                .write(&mut test_buffer, &initial_state_buffer.into_inner())?;

            let mut final_state_buffer = Cursor::new(Vec::new());

            // Write the final regs.
            MooChunkType::Registers16.write(&mut final_state_buffer, &test.final_state.regs)?;

            // Write the final queue, if not empty.
            if !test.final_state.queue.is_empty() {
                MooChunkType::QueueState.write(&mut final_state_buffer, &test.final_state.queue)?;
            }

            // Write the final ram.
            MooChunkType::Ram.write(&mut final_state_buffer, &test.final_state.ram)?;

            // Write the final state chunk.
            MooChunkType::FinalState.write(&mut test_buffer, &final_state_buffer.into_inner())?;

            let mut cycle_buffer = Cursor::new(Vec::new());
            // Write the count of cycles to the cycle buffer.
            (test.cycles.len() as u32).write_le(&mut cycle_buffer)?;
            // Write all the cycles to the cycle buffer.
            for cycle in &test.cycles {
                cycle.write(&mut cycle_buffer)?;
            }

            // Write the cycles chunk.
            MooChunkType::CycleStates.write(&mut test_buffer, &cycle_buffer.into_inner())?;

            // Create the SHA1 hash from the current state of the test buffer.
            let hash = sha1::Sha1::digest(&test_buffer.get_ref()).to_vec();

            // Write the hash chunk.
            MooChunkType::Hash.write(&mut test_buffer, &hash)?;

            // Write the test chunk.
            MooChunkType::TestHeader.write(writer, &test_buffer.into_inner())?;
        }

        Ok(())
    }
}
