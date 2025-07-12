use crate::cpu_common::{BusOp, BusOpType, BusStatusByte};
use ard808x_client::ServerCycleState;
use moo::prelude::MooCycleState;

#[derive(Clone, Debug)]
pub enum MyServerCycleState {
    State286(ServerCycleState),
}

impl MyServerCycleState {
    pub fn data_bus(&self) -> u16 {
        match self {
            MyServerCycleState::State286(state) => state.data_bus,
        }
    }
}

impl From<MyServerCycleState> for MooCycleState {
    fn from(wrapper: MyServerCycleState) -> Self {
        let MyServerCycleState::State286(state) = wrapper;

        let ale = state.bus_control_bits & 1 != 0;
        let mut pins0 = 0u8;
        if ale {
            pins0 |= MooCycleState::PIN_ALE; // Set the ALE bit
        }

        if state.pins & (MooCycleState::PIN_BHE as u16) != 0 {
            pins0 |= MooCycleState::PIN_BHE; // Set the BHE bit
        }
        if state.pins & (MooCycleState::PIN_READY as u16) != 0 {
            pins0 |= MooCycleState::PIN_READY; // Set the MRQ bit
        }
        if state.pins & (MooCycleState::PIN_LOCK as u16) != 0 {
            pins0 |= MooCycleState::PIN_LOCK; // Set the MRQ bit
        }

        let bhe = state.bus_command_bits & 0x80 != 0;
        let mut pins1 = 0;
        if bhe {
            pins1 |= 1; // Set the BHE bit
        }

        let mem_bits = !state.bus_command_bits & 0x07;
        let mut memory_status = 0;
        if mem_bits & 0b001 != 0 {
            memory_status |= 0b100; // Set the R bit
        }
        if mem_bits & 0b010 != 0 {
            memory_status |= 0b010; // Set the AW bit
        }
        if mem_bits & 0b100 != 0 {
            memory_status |= 0b001; // Set the W bit
        }

        let io_bits = (!state.bus_command_bits >> 3) & 0x07;
        let mut io_status = 0;
        if io_bits & 0b001 != 0 {
            io_status |= 0b100; // Set the R bit
        }
        if io_bits & 0b010 != 0 {
            io_status |= 0b010; // Set the AW bit
        }
        if io_bits & 0b100 != 0 {
            io_status |= 0b001; // Set the W bit
        }

        MooCycleState {
            pins0,
            address_bus: state.address_bus,
            segment: 0x3,
            memory_status,
            io_status,
            pins1,
            data_bus: state.data_bus,
            bus_state: state.cpu_status_bits & 0x0F,
            t_state: state.cpu_state_bits & 0x07,
            queue_op: 0,
            queue_byte: 0,
        }
    }
}

impl From<&MyServerCycleState> for ServerCycleState {
    fn from(wrapper: &MyServerCycleState) -> Self {
        let MyServerCycleState::State286(state) = wrapper;
        state.clone()
    }
}

impl TryFrom<&MyServerCycleState> for BusOp {
    type Error = ();

    fn try_from(wrapper: &MyServerCycleState) -> Result<Self, Self::Error> {
        match wrapper {
            MyServerCycleState::State286(state) => {
                let status_byte = BusStatusByte::V2(state.cpu_status_bits & 0x0F);
                //log::trace!("Bus status byte: {:?}", status_byte);
                if let Ok(op_type) = BusOpType::try_from(status_byte) {
                    let bus_op = BusOp {
                        op_type,
                        addr: state.address_bus,
                        bhe: state.bus_command_bits & 0x80 == 0,
                        data: state.data_bus,
                        flags: 0,
                    };
                    return Ok(bus_op);
                }
            }
        }
        Err(())
    }
}
