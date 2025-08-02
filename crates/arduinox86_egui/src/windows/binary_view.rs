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
use crate::{client::ClientContext, controls::data_table::DataTableWidget, events::GuiEventQueue};

pub struct BinaryView {
    pub name: String,
    pub icon_size: f32,
    pub dt: DataTableWidget,
}

impl Default for BinaryView {
    fn default() -> Self {
        Self {
            name: "Program".into(),
            icon_size: 18.0,
            dt: DataTableWidget::default(),
        }
    }
}

impl BinaryView {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn set_data(&mut self, data: &[u8]) {
        self.dt.set_data(data);
    }

    pub fn show(&mut self, e_ctx: &egui::Context, c_ctx: &mut ClientContext, events: &mut GuiEventQueue) {
        egui::Window::new(format!("Binary View: {}", self.name))
            .default_width(800.0)
            .default_height(600.0)
            .show(e_ctx, |ui| {
                self.dt.show(ui);
            });
    }
}
