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
    events::{GuiEvent, GuiEventQueue},
};

pub struct ClientWindow {
    pub icon_size: f32,
}

impl Default for ClientWindow {
    fn default() -> Self {
        Self { icon_size: 18.0 }
    }
}

impl ClientWindow {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn show(&mut self, e_ctx: &egui::Context, c_ctx: &mut ClientContext, events: &mut GuiEventQueue) {
        egui::Window::new("Client Connection")
            .default_width(800.0)
            .default_height(600.0)
            .show(e_ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("📤").size(self.icon_size))
                            .on_hover_text("Load registers")
                            .clicked()
                        {
                            // Do disconnect
                            events.push(GuiEvent::LoadRegisters);
                        }

                        if ui
                            .button(egui::RichText::new("⏵").size(self.icon_size))
                            .on_hover_text("Run")
                            .clicked()
                        {
                            // Do run
                        }

                        if ui
                            .button(egui::RichText::new("⏩").size(self.icon_size))
                            .on_hover_text("Run Autonomously")
                            .clicked()
                        {
                            // Do run
                        }

                        if ui
                            .button(egui::RichText::new("⏸").size(self.icon_size))
                            .on_hover_text("Pause")
                            .clicked()
                        {
                            // Do pause
                        }

                        if ui
                            .button(egui::RichText::new("🗙").size(self.icon_size))
                            .on_hover_text("Disconnect")
                            .clicked()
                        {
                            // Do disconnect
                        }
                    });

                    ui.separator();
                    ui.label(format!(
                        "Connected to {} CPU on port {}",
                        c_ctx.cpu_type.to_string(),
                        c_ctx.port_name
                    ));
                });
            });
    }
}
