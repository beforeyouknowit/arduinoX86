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
    client::ClientContext,
    controls::data_table::DataTableWidget,
    events::{GuiEvent, GuiEventQueue},
};

pub struct MemoryViewer {
    pub address_string: String,
    pub address: u32,
    pub size_string: String,
    pub size: u32,
    pub icon_size: f32,
    pub dt: DataTableWidget,
}

impl Default for MemoryViewer {
    fn default() -> Self {
        Self {
            address_string: "00000000".to_string(),
            address: 0,
            size_string: "00001000".to_string(), // Default to 4KB
            size: 0x1000,                        // Default to 4KB
            icon_size: 24.0,
            dt: DataTableWidget::default(),
        }
    }
}

impl MemoryViewer {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn set_data(&mut self, data: &[u8]) {
        self.dt.set_data(data);
    }

    pub fn show(&mut self, e_ctx: &egui::Context, c_ctx: &mut ClientContext, events: &mut GuiEventQueue) {
        egui::Window::new("Memory Viewer")
            .default_width(800.0)
            .default_height(600.0)
            .show(e_ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("📥").size(self.icon_size))
                            .on_hover_text("Download Memory")
                            .clicked()
                        {
                            events.push(GuiEvent::ReadMemory {
                                address: self.address,
                                size:    self.size,
                            });
                        }

                        ui.label("Address:");
                        if ui.text_edit_singleline(&mut self.address_string).lost_focus() {
                            if let Ok(addr) = u32::from_str_radix(&self.address_string, 16) {
                                self.address = addr;
                            }
                            else {
                                self.address = 0;
                            }
                        }
                        ui.label("Size:");
                        if ui.text_edit_singleline(&mut self.size_string).lost_focus() {
                            if let Ok(size) = u32::from_str_radix(&self.size_string, 16) {
                                self.size = size;
                            }
                            else {
                                self.size = 0;
                            }
                        }
                    });

                    self.dt.show(ui);
                });
            });
    }
}
