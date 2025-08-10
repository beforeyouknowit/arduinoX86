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
use crate::{client::ClientContext, events::GuiEventQueue, structs::BinaryBlob, windows::BinaryView};
use anyhow::Result;

#[derive(Default)]
pub struct ResourceManager {
    blobs: Vec<BinaryBlob>,
    blob_windows: Vec<BinaryView>,
}

impl ResourceManager {
    pub fn blob_exists(&self, blob_name: &str) -> bool {
        self.blobs.iter().any(|b| b.name == blob_name)
    }

    pub fn add_blob(&mut self, blob: BinaryBlob) -> Result<()> {
        if self.blob_exists(&blob.name) {
            return Err(anyhow::anyhow!("Blob with name '{}' already exists.", blob.name));
        }
        self.blobs.push(blob.clone());
        let mut view = BinaryView::new(&blob.name);

        view.set_data(&blob.data);

        self.blob_windows.push(view);
        Ok(())
    }

    pub fn remove_blob(&mut self, blob_name: &str) -> Result<()> {
        if !self.blob_exists(blob_name) {
            return Err(anyhow::anyhow!("Blob with name '{}' does not exist.", blob_name));
        }
        if let Some(index) = self.blobs.iter().position(|b| b.name == blob_name) {
            self.blobs.remove(index);
            self.blob_windows.remove(index);
        }
        Ok(())
    }

    pub fn show(&mut self, e_ctx: &egui::Context, c_ctx: &mut ClientContext, events: &mut GuiEventQueue) {
        for window in &mut self.blob_windows {
            window.show(e_ctx, c_ctx, events);
        }
    }

    pub fn blobs(&self) -> &[BinaryBlob] {
        &self.blobs
    }
}
