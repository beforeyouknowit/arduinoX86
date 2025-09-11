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
use crate::{structs::BinaryBlob, windows::BinaryView};

use anyhow::Result;

#[derive(Default)]
pub struct ResourceManager {
    blobs: Vec<BinaryBlob>,
}

impl ResourceManager {
    pub fn blob_exists(&self, blob_name: &str) -> bool {
        self.blobs.iter().any(|b| b.name == blob_name)
    }

    pub fn add_blob(&mut self, blob: BinaryBlob) -> Result<BinaryView> {
        if self.blob_exists(&blob.name) {
            return Err(anyhow::anyhow!("Blob with name '{}' already exists.", blob.name));
        }
        self.blobs.push(blob.clone());
        let mut view = BinaryView::new(&blob.name);
        view.set_data(&blob.data);
        Ok(view)
    }

    pub fn update_blob(&mut self, blob_name: &str, new_data: &[u8]) -> Result<()> {
        if !self.blob_exists(blob_name) {
            return Err(anyhow::anyhow!("Blob with name '{}' does not exist.", blob_name));
        }
        if let Some(blob) = self.blobs.iter_mut().find(|b| b.name == blob_name) {
            blob.data = new_data.to_vec();
        }
        Ok(())
    }

    pub fn remove_blob(&mut self, blob_name: &str) -> Result<()> {
        if !self.blob_exists(blob_name) {
            return Err(anyhow::anyhow!("Blob with name '{}' does not exist.", blob_name));
        }
        if let Some(index) = self.blobs.iter().position(|b| b.name == blob_name) {
            self.blobs.remove(index);
        }
        Ok(())
    }

    pub fn blob(&self, blob_name: &str) -> Option<&BinaryBlob> {
        self.blobs.iter().find(|b| b.name == blob_name)
    }

    pub fn blob_mut(&mut self, blob_name: &str) -> Option<&mut BinaryBlob> {
        self.blobs.iter_mut().find(|b| b.name == blob_name)
    }

    pub fn blobs(&self) -> &[BinaryBlob] {
        &self.blobs
    }
}
