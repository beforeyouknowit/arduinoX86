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
use egui::TextBuffer;
use egui_extras::syntax_highlighting::SyntectSettings;
use syntect::parsing::{SyntaxSet, SyntaxSetBuilder};

pub struct CodeEditor {
    language: String,
    code: String,
    open: bool,
    syntect_settings: SyntectSettings,
}

impl Default for CodeEditor {
    fn default() -> Self {
        CodeEditor::new()
    }
}

impl CodeEditor {
    pub fn new() -> Self {
        let mut ss_builder = SyntaxSetBuilder::new();
        ss_builder
            .add_from_folder("./syntax", true)
            .expect("failed to load syntax definitions");

        for syntax in ss_builder.syntaxes() {
            log::debug!("Loaded syntax: {}", syntax.name);
        }

        let syntax_set = ss_builder.build();

        Self {
            language: "asm".to_string(),
            code: "\
                nop\n\
                mov ax, 0x1234\n\
                hlt\n"
                .to_string(),
            open: false,
            syntect_settings: SyntectSettings {
                ps: syntax_set,
                ts: syntect::highlighting::ThemeSet::load_defaults(),
            },
        }
    }

    pub fn open(&self) -> &bool {
        &self.open
    }

    pub fn open_mut(&mut self) -> &mut bool {
        &mut self.open
    }

    pub fn show(&mut self, e_ctx: &egui::Context) {
        if !self.open {
            return;
        }
        egui::Window::new("Code Editor")
            .default_width(400.0)
            .default_height(300.0)
            .show(e_ctx, |ui| {
                let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

                let mut layouter = |ui: &egui::Ui, buf: &dyn TextBuffer, wrap_width: f32| {
                    let mut layout_job = egui_extras::syntax_highlighting::highlight_with(
                        ui.ctx(),
                        ui.style(),
                        &theme,
                        buf.as_str(),
                        &self.language,
                        &self.syntect_settings,
                    );
                    layout_job.wrap.max_width = wrap_width;
                    ui.fonts(|f| f.layout_job(layout_job))
                };

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.code)
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .desired_rows(10)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter),
                    );
                });
            });
    }
}
