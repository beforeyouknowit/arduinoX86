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
use arduinox86_client::ServerCycleState;
use egui_extras::{Column, TableBuilder};

#[derive(Default)]
pub struct CycleTable {
    cycles: Vec<ServerCycleState>,
}

impl CycleTable {
    pub fn new() -> Self {
        CycleTable { cycles: Vec::new() }
    }

    pub fn set_cycles(&mut self, cycles: Vec<ServerCycleState>) {
        self.cycles = cycles;
    }

    pub fn cycles(&self) -> &[ServerCycleState] {
        &self.cycles
    }

    pub fn clear(&mut self) {
        self.cycles.clear();
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        if self.cycles.is_empty() {
            ui.label("No cycles available");
            return;
        }

        let available_height = ui.available_height();
        let num_rows = self.cycles.len();
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::remainder())
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("ALE");
                });
                header.col(|ui| {
                    ui.strong("Address Bus");
                });
                header.col(|ui| {
                    ui.strong("Data Bus");
                });
            })
            .body(|mut body| {
                body.rows(text_height, num_rows, |mut row| {
                    let idx = row.index();
                    row.col(|ui| {
                        ui.label(egui::RichText::new("A:").monospace());
                    });
                    row.col(|ui| {
                        ui.label(egui::RichText::new(format!("{:08X}", self.cycles[idx].address_bus)).monospace());
                    });
                    row.col(|ui| {
                        ui.label(egui::RichText::new(format!("{:04X}", self.cycles[idx].data_bus)).monospace());
                    });
                });
            });
    }
}
