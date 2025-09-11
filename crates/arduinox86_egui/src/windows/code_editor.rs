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
use crate::events::{GuiEvent, GuiEventQueue};
use egui::TextBuffer;
use egui_extras::syntax_highlighting::SyntectSettings;

pub struct CodeEditor {
    icon_size: f32,
    program_name: String,
    language: String,
    code: String,
    open: bool,

    assembler_output: String,
}

impl CodeEditor {
    pub fn new(program_name: &str) -> Self {
        Self {
            icon_size: 24.0,
            program_name: program_name.to_string(),
            language: "asm".to_string(),
            code: "hlt\n".to_string(),
            open: false,
            assembler_output: "".to_string(),
        }
    }

    pub fn open(&self) -> &bool {
        &self.open
    }

    pub fn open_mut(&mut self) -> &mut bool {
        &mut self.open
    }

    pub fn program_name(&self) -> &str {
        &self.program_name
    }

    pub fn set_assembler_output(&mut self, output: &str) {
        self.assembler_output = output.to_string();
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn set_code(&mut self, code: String) {
        self.code = code;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, syntect_settings: &SyntectSettings, events: &mut GuiEventQueue) {
        if !self.open {
            return;
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(egui::RichText::new(format!("{}", egui_phosphor::regular::CALCULATOR)).size(self.icon_size))
                    .on_hover_text("Assemble")
                    .clicked()
                {
                    events.push(GuiEvent::AssembleProgram {
                        program_name: self.program_name.clone(),
                    });
                }
            });

            ui.horizontal(|ui| {
                ui.label("Program Name:");
                ui.text_edit_singleline(&mut self.program_name);
            });

            ui.separator();
        });

        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

        let mut layouter = |ui: &egui::Ui, buf: &dyn TextBuffer, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight_with(
                ui.ctx(),
                ui.style(),
                &theme,
                buf.as_str(),
                &self.language,
                syntect_settings,
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        let mut max_scroll_height = ui.available_rect_before_wrap().height();
        if !self.assembler_output.is_empty() {
            max_scroll_height -= 200.0; // Reserve space for assembler output
        }

        egui::ScrollArea::vertical()
            .max_height(max_scroll_height)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(40)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });

        if !self.assembler_output.is_empty() {
            ui.separator();

            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Assembler Output:");
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui
                        .button(egui::RichText::new("❌").size(18.0))
                        .on_hover_text("Clear")
                        .clicked()
                    {
                        self.assembler_output = "".to_string();
                    }
                });
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("error")
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.assembler_output.as_str())
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .desired_rows(20)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY),
                    );
                });
        }
    }
}
