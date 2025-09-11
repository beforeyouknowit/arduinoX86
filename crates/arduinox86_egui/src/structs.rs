/*
    ArduinoX86 Copyright 2022-2025 Daniel Balsom
    https://github.com/dbalsom/arduinoX86

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of this software and associated documentation files (the “Software”),
    to deal in the Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, sublicense,
    and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
*/
use crate::{
    enums::{BinaryBlobType, MountAddress, ScheduleType},
    events::GuiEvent,
};
use std::path::Path;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct BinaryBlob {
    pub name: String,
    pub mount_address: MountAddress,
    pub blob_type: BinaryBlobType,
    pub data: Vec<u8>,
}

impl BinaryBlob {
    pub fn new(name: String, mount_address: MountAddress, blob_type: BinaryBlobType, data: Vec<u8>) -> Self {
        Self {
            name,
            mount_address,
            blob_type,
            data,
        }
    }

    pub fn from_path(
        name: &str,
        mount_address: MountAddress,
        blob_type: BinaryBlobType,
        path: impl AsRef<Path>,
    ) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let data = std::fs::read(path)?;
        Ok(Self {
            name: name.to_string(),
            mount_address,
            blob_type,
            data,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mount_address(&self) -> MountAddress {
        self.mount_address
    }
    pub fn set_mount_address(&mut self, mount_address: MountAddress) {
        self.mount_address = mount_address;
    }
}

pub struct ScheduledEvent {
    pub s_type: ScheduleType,
    pub event: GuiEvent,
    pub time: u64,     // time to trigger in milliseconds
    pub ms_accum: u64, // accumulated time in milliseconds
}

impl ScheduledEvent {
    pub fn new(s_type: ScheduleType, event: GuiEvent, time: u64) -> Self {
        Self {
            s_type,
            event,
            time,
            ms_accum: 0,
        }
    }
}
