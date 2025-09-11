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
    events::{GuiEvent, GuiEventQueue},
    widgets::cycle_display::CycleDisplay,
};
use arduinox86_client::{ServerCpuType, ServerCycleLogPrinter, ServerCycleState};
use egui::{Color32, Margin, Response};

#[derive(Default)]
pub struct CycleTable {
    arch: ServerCpuType,
    cycles: Vec<ServerCycleState>,
    data_bus_str: String,
    address_latch: u32,
}

impl CycleTable {
    pub fn new(arch: ServerCpuType) -> Self {
        CycleTable {
            arch,
            cycles: Vec::new(),
            data_bus_str: String::new(),
            address_latch: 0,
        }
    }

    pub fn set_arch(&mut self, arch: ServerCpuType) {
        self.arch = arch;
    }

    pub fn data_bus_str(&self) -> &str {
        &self.data_bus_str
    }

    pub fn set_cycles(&mut self, cycles: Vec<ServerCycleState>) {
        if cycles.is_empty() {
            self.data_bus_str.clear();
        }
        else {
            let last_cycle = cycles.last().unwrap();
            self.data_bus_str = format!("{:04X}", last_cycle.data_bus);
        }
        self.cycles = cycles;
    }

    pub fn push_cycle(&mut self, cycle: ServerCycleState) {
        self.data_bus_str = format!("{:04X}", cycle.data_bus);
        self.cycles.push(cycle);
    }

    pub fn cycles(&self) -> &[ServerCycleState] {
        &self.cycles
    }

    pub fn clear(&mut self) {
        self.cycles.clear();
    }

    pub fn show(&mut self, ui: &mut egui::Ui, events: &mut GuiEventQueue) -> Option<Response> {
        if self.cycles.is_empty() {
            ui.label("No cycles available");
            return None;
        }

        self.address_latch = 0;

        // let available_height = ui.available_height();
        // let num_rows = self.cycles.len();
        // let text_height = egui::TextStyle::Body
        //     .resolve(ui.style())
        //     .size
        //     .max(ui.spacing().interact_size.y);

        let bg: Color32 = ui.visuals().code_bg_color;
        let max_scroll_height = 400.0;

        //let rect = ui.max_rect();
        //ui.painter().rect_filled(rect, 0.0, bg);

        let mut inner_response = None;

        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.horizontal(|ui| {
                ui.label("Cycle Output:");
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui
                    .button(egui::RichText::new(format!("{}", egui_phosphor::regular::X)).size(18.0))
                    .on_hover_text("Clear")
                    .clicked()
                {
                    events.push(GuiEvent::ClearCycleLog);
                    self.cycles.clear();
                }
                if ui
                    .button(egui::RichText::new(format!("{}", egui_phosphor::regular::CLIPBOARD_TEXT)).size(18.0))
                    .on_hover_text("Copy")
                    .clicked()
                {
                    let printer = ServerCycleLogPrinter::new(self.arch, &self.cycles);
                    ui.ctx().copy_text(printer.to_string());
                }
            });
        });

        egui::Frame::new()
            .fill(bg)
            .inner_margin(Margin::same(6))
            .outer_margin(Margin::ZERO)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(max_scroll_height)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        let num_cycles = self.cycles.len();
                        for (i, cycle) in self.cycles.iter().enumerate() {
                            let mut data_str_opt = None;
                            // If is last cycle
                            if (i == num_cycles - 1) && cycle.is_reading() {
                                data_str_opt = Some(&mut self.data_bus_str);
                            }

                            let cycle_display =
                                CycleDisplay::new(self.arch, cycle.clone(), &mut self.address_latch, data_str_opt);

                            inner_response = Some(ui.add(cycle_display));
                        }
                    });
            });

        if let Some(resp) = inner_response.as_ref() {
            if resp.changed() {
                log::debug!("ClientWindow::show(): Response: {:?}", resp);
            }
        }

        inner_response
    }
}
