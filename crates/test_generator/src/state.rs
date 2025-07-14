use crate::cpu_common::{BusOp, BusOpType};
use anyhow::bail;
use ard808x_client::CpuWidth;
use indexmap::IndexMap;

pub struct InitialState {
    pub initial_state: IndexMap<u32, u8>,
    pub initial_ram: Vec<[u32; 2]>,
}

/// Try to calculate the initial memory state from a list of Bus operations.
pub fn initial_state_from_ops(
    cpu_width: CpuWidth,
    cs_base: u32,
    ip: u16,
    instr_bytes: &[u8],
    prefetch_len: usize,
    all_ops: &Vec<BusOp>,
) -> anyhow::Result<InitialState> {
    let mut initial_state: IndexMap<u32, u8> = IndexMap::new();
    let mut code_addresses: IndexMap<u32, (u8, bool)> = IndexMap::new();

    // Add the instruction bytes to the initial state. They cannot be modified
    // by the validated instruction because every instruction is done fetching
    // operands by the time it does any writes, so they had to be in the
    // initial state.
    let mut pc = ip as u32;

    match cpu_width {
        CpuWidth::Eight => {
            for byte in instr_bytes {
                let flat_addr = cs_base + pc;
                let data_value = *byte;

                code_addresses.insert(flat_addr, (data_value, true));
                initial_state.insert(flat_addr, data_value);
                pc = pc.wrapping_add(1);
            }

            // If the instruction is shorter than the prefetch length, add NOPs to the initial state
            if prefetch_len > instr_bytes.len() {
                for _ in 0..(prefetch_len - instr_bytes.len()) {
                    let flat_addr = cs_base + pc;
                    code_addresses.insert(flat_addr, (0x90, true));
                    initial_state.insert(flat_addr, 0x90);
                    pc = pc.wrapping_add(1);
                }
            }
        }
        CpuWidth::Sixteen => {
            let mut flat_addr = cs_base + pc;

            let mut skip_bytes = 0;
            if flat_addr & 1 != 0 {
                // Odd address. Add the first fetch with the low byte masked off.
                log::trace!(
                    "Inserting {:02X} at odd address {:05X}",
                    instr_bytes[0],
                    flat_addr
                );
                code_addresses.insert(flat_addr, (instr_bytes[0], true));
                initial_state.insert(flat_addr, instr_bytes[0]);
                pc = pc.wrapping_add(1);
                skip_bytes = 1;
            }

            for chunk in instr_bytes[skip_bytes..].chunks(2) {
                flat_addr = cs_base + pc;
                if chunk.len() == 2 {
                    let word = chunk[0] as u16 | ((chunk[1] as u16) << 8);
                    log::trace!("Inserting {:04X} at address {:05X}", word, flat_addr);
                    code_addresses.insert(flat_addr, (word as u8, true));
                    initial_state.insert(flat_addr, word as u8);
                    code_addresses.insert(flat_addr.wrapping_add(1), ((word >> 8) as u8, true));
                    initial_state.insert(flat_addr.wrapping_add(1), (word >> 8) as u8);
                    pc = pc.wrapping_add(2);
                } else {
                    // Last byte, so just insert it as a single byte.
                    let word = chunk[0] as u16;
                    log::trace!("Inserting {:04X} at address {:05X}", word, flat_addr);
                    code_addresses.insert(flat_addr, (word as u8, true));
                    initial_state.insert(flat_addr, word as u8);
                    pc = pc.wrapping_add(1);
                }
            }

            // If the instruction is shorter than the prefetch length, add NOPs to the initial state
            if prefetch_len > instr_bytes.len() {
                for _ in 0..(prefetch_len - instr_bytes.len()) {
                    let flat_addr = cs_base + pc;
                    code_addresses.insert(flat_addr, (0x90, true));
                    initial_state.insert(flat_addr, 0x90);
                    pc = pc.wrapping_add(1);
                }
            }
        }
    }

    let mut shadowed_addresses: IndexMap<u32, bool> = IndexMap::new();
    let mut read_addresses: IndexMap<u32, u16> = IndexMap::new();
    let mut write_addresses: IndexMap<u32, u16> = IndexMap::new();

    for op in all_ops {
        match op.op_type {
            BusOpType::MemRead => {
                for (addr, data) in bytes_from_bus_op(op).into_iter() {
                    read_addresses.insert(addr, data as u16);
                }

                if write_addresses.get(&op.addr).is_some() {
                    // Reading from an address the instruction wrote to (not sure if this ever happens?)
                    // In any case, don't add this to the initial state since it happened after a write.
                    log::debug!(
                        "Reading from written address: [{:05X}]:{:02X}!",
                        op.addr,
                        op.data
                    );
                } else {
                    // This address was never written to, so the value here must have been part of the
                    // initial state.
                    for (addr, data) in bytes_from_bus_op(op).into_iter() {
                        initial_state.insert(addr, data);
                    }
                }
            }
            BusOpType::CodeRead => {
                for (addr, op_data) in bytes_from_bus_op(op).into_iter() {
                    if let Some((data, flag)) = code_addresses.get(&addr) {
                        if *flag {
                            // This operation corresponds to an initial fetch.
                            // Just as a sanity check, compare bytes.

                            log::trace!(
                                "Validating initial instruction fetch: [{:05X}]:{:04X} with data {:04X}",
                                addr,
                                op_data,
                                data
                            );

                            if *data != op_data {
                                bail!(
                                    "Initial instruction fetch mismatch at [{:05X}]: expected {:02X}, got {:02X}",
                                    addr,
                                    data,
                                    op_data
                                );
                            }
                            //log::debug!("Validated initial instruction fetch: [{:05X}]:{:02X}", op.addr, op.data);
                        } else {
                            // How can we be fetching the same byte twice?
                            bail!("Illegal duplicate fetch!");
                        }
                    } else {
                        // Fetch outside of instruction boundaries.

                        // Check if we are fetching from a shadowed address.
                        if shadowed_addresses.get(&addr).is_some() {
                            // We are fetching from an address we wrote to and don't know the value of.
                            log::debug!(
                                "Detected self modifying code! Fetch from: [{:05X}] was written to by BusOp.",
                                addr
                            );

                            initial_state.insert(addr, op_data);

                            // // Initial state would have been NOP.
                            // match cpu_width {
                            //     CpuWidth::Eight => {
                            //         initial_state.insert(op.addr, 0x90);
                            //         code_addresses.insert(op.addr, (0x90, false));
                            //     }
                            //     CpuWidth::Sixteen => {
                            //         if op.addr & 1 == 1 {
                            //             // Odd address, so insert a single NOP.
                            //             initial_state.insert(op.addr, 0x90);
                            //             code_addresses.insert(op.addr, (0x90, false));
                            //         } else {
                            //             // Even address, so insert a NOP word.
                            //             initial_state.insert(op.addr, 0x90);
                            //             initial_state.insert(op.addr.wrapping_add(1), 0x90);
                            //             code_addresses.insert(op.addr, (0x90, false));
                            //         }
                            //     }
                            // }
                        } else {
                            // Address wasn't shadowed, so safe to add this fetch to the initial state.
                            //log::debug!("Adding subsequent instruction fetch to initial state [{:05X}]:{:02X}", op.addr, op.data);
                            initial_state.insert(addr, op_data);
                        }
                    }
                }
            }
            BusOpType::MemWrite => {
                for (addr, data) in bytes_from_bus_op(op).into_iter() {
                    // Check if this address was read from previously.
                    if read_addresses.get(&addr).is_some() || code_addresses.get(&addr).is_some() {
                        // Modifying a previously read address. This is fine.
                    } else {
                        // This address was never read from, so this write shadows
                        // the original value at this address. Mark it as a
                        // shadowed address.
                        shadowed_addresses.insert(addr, true);

                        // Since this isn't a fetch, we don't have to add it to the initial state
                        // - whatever it was isn't important
                    }
                    write_addresses.insert(addr, data as u16);
                }
            }
            _ => {}
        }
    }

    // Collapse initial state hash into vector of arrays
    let ram_vec: Vec<[u32; 2]> = initial_state
        .iter()
        .map(|(&addr, &data)| [addr, data as u32])
        .collect();

    // v2: Don't sort the initial ram vector; leave it in order of operation
    //ram_vec.sort_by(|a, b| a[0].cmp(&b[0]));

    Ok(InitialState {
        initial_state,
        initial_ram: ram_vec,
    })
}

pub fn bytes_from_bus_op(op: &BusOp) -> Vec<(u32, u8)> {
    let mut bytes = Vec::new();
    let mut high_offset = 0;
    if op.addr & 1 == 0 {
        // Even address, so push the low byte.
        bytes.push((op.addr, (op.data & 0xFF) as u8));
        high_offset = 1;
    }
    if op.bhe {
        // BHE is set, so push the high byte also.
        bytes.push((op.addr + high_offset, ((op.data >> 8) & 0xFF) as u8));
    }
    bytes
}

pub fn final_state_from_ops(
    initial_state: IndexMap<u32, u8>,
    all_ops: &[BusOp],
) -> anyhow::Result<Vec<[u32; 2]>> {
    let mut ram_ops = all_ops.to_vec();
    // We modify the initial state by inserting write operations into it.
    let mut final_state = initial_state.clone();

    // Filter out IO reads, these are not used for ram setup
    ram_ops.retain(|&op| !matches!(op.op_type, BusOpType::IoRead));
    // Filter out IO writes, these are not used for ram setup
    ram_ops.retain(|&op| !matches!(op.op_type, BusOpType::IoWrite));

    let mut write_addresses: IndexMap<u32, u16> = IndexMap::new();
    //let mut ram_hash: HashMap<u32, u8> = HashMap::new();

    for op in ram_ops {
        match op.op_type {
            BusOpType::MemRead => {
                // Check if this read is already in memory. If it is, it must have the same value,
                // or we are out of sync!
                for (addr, data) in bytes_from_bus_op(&op).into_iter() {
                    match initial_state.get(&addr) {
                        Some(d) => {
                            if *d != data {
                                // Read op doesn't match initial state. Invalid!
                                bail!(
                                    "Memop sync fail. MemRead [{:05X}]:{:02X}, hash value: {:02X}",
                                    addr,
                                    data,
                                    d
                                );
                            }
                        }
                        None => {
                            // Read from mem op not in initial state. If we didn't write to this value, this read is invalid.
                            if write_addresses.get(&addr).is_some() {
                                // Ok, we wrote to this address at some point, so we can read it even if it wasn't in the
                                // initial state.
                            } else {
                                // We never wrote to this address, and it's not in the initial state. This is invalid!
                                bail!("Memop sync fail. MemRead from address not in initial state and not written: [{:05X}]:{:02X}", addr, data);
                            }
                        }
                    }
                }
            }
            BusOpType::MemWrite => {
                // No need to check writes; just insert the values.
                write_addresses.insert(op.addr, op.data);
                for (addr, data) in bytes_from_bus_op(&op).into_iter() {
                    final_state.insert(addr, data);
                }
            }
            _ => {}
        }
    }

    // Collapse ram hash into vector of arrays
    let mut ram_vec: Vec<[u32; 2]> = final_state
        .iter()
        .map(|(&addr, &data)| [addr, data as u32])
        .collect();

    // Remove entries from final RAM vector that are present in the initial state.
    ram_vec.retain(|&[addr, data]| {
        // Keep values where the address is not found, or the data is different.
        match initial_state.get(&addr) {
            Some(&initial_data) => initial_data != (data as u8),
            None => true, // If the address is not in the initial state, keep it.
        }
    });

    // v2: Don't sort the final ram vector. Leave in order of operation.
    //ram_vec.sort_by(|a, b| a[0].cmp(&b[0]));

    Ok(ram_vec)
}
