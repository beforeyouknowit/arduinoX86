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
    enums::{Register16, Register32},
    events::GuiEventQueue,
    register_state::RegisterStringStateV3,
};
use arduinox86_client::{Registers32, RemoteCpuRegistersV3, RemoteCpuRegistersV3A, ServerCpuType};
use egui::{show_tooltip, TextBuffer};

const COLUMN_WIDTH: f32 = 150.0;

#[derive(Default)]
pub struct RegisterControlV3 {
    pub regs: RemoteCpuRegistersV3,
    pub reg_strings: RegisterStringStateV3,
    pub reg_updated: bool,
    pub flag_updated: bool,
}

impl RegisterControlV3 {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn set_regs(&mut self, regs: &RemoteCpuRegistersV3) {
        self.regs = regs.clone();
        self.reg_strings = RegisterStringStateV3::from(&self.regs);
    }

    pub fn regs(&self) -> &RemoteCpuRegistersV3 {
        &self.regs
    }

    #[rustfmt::skip]
    pub fn show(&mut self, ui: &mut egui::Ui, events: &mut GuiEventQueue) {

        match &mut self.regs {
            RemoteCpuRegistersV3::A(_) | RemoteCpuRegistersV3::B(_) => {
                self.show_regs32(ui, events);
            }
            _ => {}
        }

        egui::Grid::new("reg_flags")
            .striped(true)
            .max_col_width(10.0)
            .show(ui, |ui| {

                Self::show_flagbit_mut(ui, &mut self.reg_strings.flags.o_fl, &mut self.flag_updated, "O", "Overflow");
                Self::show_flagbit_mut(ui, &mut self.reg_strings.flags.d_fl, &mut self.flag_updated, "D", "Direction");
                Self::show_flagbit_mut(ui, &mut self.reg_strings.flags.i_fl, &mut self.flag_updated,"I","Interrupt enable",);
                Self::show_flagbit_mut(ui, &mut self.reg_strings.flags.t_fl, &mut self.flag_updated, "T", "Trap");
                Self::show_flagbit_mut(ui, &mut self.reg_strings.flags.s_fl, &mut self.flag_updated, "S", "Sign");
                Self::show_flagbit_mut(ui, &mut self.reg_strings.flags.z_fl, &mut self.flag_updated, "Z", "Zero");
                Self::show_flagbit_mut(ui, &mut self.reg_strings.flags.a_fl, &mut self.flag_updated, "A","Auxiliary carry",);
                Self::show_flagbit_mut(ui, &mut self.reg_strings.flags.p_fl, &mut self.flag_updated, "P", "Parity");
                Self::show_flagbit_mut(ui, &mut self.reg_strings.flags.c_fl, &mut self.flag_updated, "C", "Carry");
                ui.end_row();
            });
    }

    fn show_regs32(&mut self, ui: &mut egui::Ui, events: &mut GuiEventQueue) {
        egui::Grid::new("reg_general_grid")
            .striped(true)
            .min_col_width(COLUMN_WIDTH)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    Self::show_reg_mut32(
                        ui,
                        "EAX",
                        &mut self.reg_strings.eax,
                        Register32::EAX,
                        Registers32::eax_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.horizontal(|ui| {
                    Self::show_reg_mut32(
                        ui,
                        "ESP",
                        &mut self.reg_strings.esp,
                        Register32::ESP,
                        Registers32::esp_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.end_row();

                ui.horizontal(|ui| {
                    Self::show_reg_mut32(
                        ui,
                        "EBX",
                        &mut self.reg_strings.ebx,
                        Register32::EBX,
                        Registers32::ebx_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.horizontal(|ui| {
                    Self::show_reg_mut32(
                        ui,
                        "EBP",
                        &mut self.reg_strings.ebp,
                        Register32::EBP,
                        Registers32::ebp_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.end_row();

                ui.horizontal(|ui| {
                    Self::show_reg_mut32(
                        ui,
                        "ECX",
                        &mut self.reg_strings.ecx,
                        Register32::ECX,
                        Registers32::ecx_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.horizontal(|ui| {
                    Self::show_reg_mut32(
                        ui,
                        "ESI",
                        &mut self.reg_strings.esi,
                        Register32::ESI,
                        Registers32::esi_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.end_row();

                ui.horizontal(|ui| {
                    Self::show_reg_mut32(
                        ui,
                        "EDX",
                        &mut self.reg_strings.edx,
                        Register32::EDX,
                        Registers32::edx_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.horizontal(|ui| {
                    Self::show_reg_mut32(
                        ui,
                        "EDI",
                        &mut self.reg_strings.edi,
                        Register32::EDI,
                        Registers32::edi_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.end_row();
            });

        ui.separator();

        egui::Grid::new("reg_segment")
            .striped(true)
            .min_col_width(COLUMN_WIDTH)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    Self::show_reg_mut16(
                        ui,
                        "DS ",
                        &mut self.reg_strings.ds,
                        Register16::DS,
                        Registers32::ds_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.end_row();
                ui.horizontal(|ui| {
                    Self::show_reg_mut16(
                        ui,
                        "ES ",
                        &mut self.reg_strings.es,
                        Register16::ES,
                        Registers32::es_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.end_row();
                ui.horizontal(|ui| {
                    Self::show_reg_mut16(
                        ui,
                        "FS ",
                        &mut self.reg_strings.fs,
                        Register16::FS,
                        Registers32::fs_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.end_row();
                ui.horizontal(|ui| {
                    Self::show_reg_mut16(
                        ui,
                        "GS ",
                        &mut self.reg_strings.gs,
                        Register16::GS,
                        Registers32::gs_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.end_row();
                ui.horizontal(|ui| {
                    Self::show_reg_mut16(
                        ui,
                        "SS ",
                        &mut self.reg_strings.ss,
                        Register16::SS,
                        Registers32::ss_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.end_row();
                ui.horizontal(|ui| {
                    Self::show_reg_mut16(
                        ui,
                        "CS ",
                        &mut self.reg_strings.cs,
                        Register16::CS,
                        Registers32::cs_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.horizontal(|ui| {
                    Self::show_reg_mut32(
                        ui,
                        "EIP",
                        &mut self.reg_strings.eip,
                        Register32::EIP,
                        Registers32::eip_mut(&mut self.regs),
                        &mut self.reg_updated,
                        events,
                    );
                });
                ui.end_row();

                // ui.horizontal(|ui| {
                //     // IP is not a real register - don't allow editing (?)
                //     ui.label(egui::RichText::new("IP:").text_style(egui::TextStyle::Monospace));
                //     ui.add(
                //         egui::TextEdit::singleline(&mut self.cpu_state.ip.as_str()).font(egui::TextStyle::Monospace),
                //     );
                // });
            });

        ui.separator();
    }

    fn show_reg_mut32(
        ui: &mut egui::Ui,
        label: &str,
        reg_string: &mut String,
        reg_id: Register32,
        reg_mut: &mut u32,
        updated: &mut bool,
        events: &mut GuiEventQueue,
    ) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(label).text_style(egui::TextStyle::Monospace));
            let response = ui.add(
                egui::TextEdit::singleline(reg_string)
                    .char_limit(8)
                    .font(egui::TextStyle::Monospace),
            );

            if response.lost_focus() {
                // TextEdit loses focus on enter or tab. In any case, we'll apply the value if it is valid.
                match u32::from_str_radix(reg_string.as_str(), 16) {
                    Ok(val) => {
                        log::debug!("Register {:?} updated to 0x{:04X}", reg_id, val);
                        *reg_mut = val;
                        *reg_string = format!("{:08X}", val);
                        //events.send(GuiEvent::Register16Update(reg, val));
                    }
                    Err(_) => {
                        // Invalid value
                        log::warn!("Invalid value for register {}: {}", label, reg_string);
                        *reg_string = "00000000".to_string(); // Reset to 0 if invalid
                    }
                }
                *updated = true;
            }
        });
    }

    fn show_reg_mut16(
        ui: &mut egui::Ui,
        label: &str,
        reg_string: &mut String,
        reg_id: Register16,
        reg_mut: &mut u16,
        updated: &mut bool,
        events: &mut GuiEventQueue,
    ) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(label).text_style(egui::TextStyle::Monospace));
            let response = ui.add(
                egui::TextEdit::singleline(reg_string)
                    .char_limit(4)
                    .font(egui::TextStyle::Monospace),
            );

            if response.lost_focus() {
                // TextEdit loses focus on enter or tab. In any case, we'll apply the value if it is valid.
                match u16::from_str_radix(reg_string.as_str(), 16) {
                    Ok(val) => {
                        log::debug!("Register {:?} updated to 0x{:04X}", reg_id, val);
                        *reg_mut = val;
                        *reg_string = format!("{:04X}", val);
                        //events.send(GuiEvent::Register16Update(reg, val));
                    }
                    Err(_) => {
                        // Invalid value - could change text color to red?
                        log::warn!("Invalid value for register {}: {}", label, reg_string);
                        *reg_string = "0000".to_string(); // Reset to 0 if invalid
                    }
                }
                *updated = true;
            }
        });
    }

    /// Display a widget for a flag bit. It will show the provided tooltip text on hover.
    fn show_flagbit(ui: &mut egui::Ui, text: &mut dyn TextBuffer, label: &str, tip: &str) {
        ui.vertical(|ui| {
            ui.add(
                egui::TextEdit::singleline(text)
                    .char_limit(1)
                    .horizontal_align(egui::Align::Center)
                    .font(egui::TextStyle::Monospace),
            );
            ui.centered_and_justified(|ui| {
                if ui
                    .add(
                        egui::Label::new(egui::RichText::new(label).text_style(egui::TextStyle::Monospace))
                            .selectable(false),
                    )
                    .hovered()
                {
                    show_tooltip(ui.ctx(), ui.layer_id(), egui::Id::new("flag_tooltip"), |ui| {
                        ui.horizontal(|ui| {
                            ui.label(tip);
                        });
                    });
                }
            });
        });
    }

    /// Display a widget for an editable flag bit. It will show the provided tooltip text on hover.
    fn show_flagbit_mut(ui: &mut egui::Ui, text: &mut String, updated: &mut bool, label: &str, tip: &str) {
        ui.vertical(|ui| {
            let edit_response = ui.add(
                egui::TextEdit::singleline(text)
                    .char_limit(1)
                    .horizontal_align(egui::Align::Center)
                    .char_limit(1)
                    .font(egui::TextStyle::Monospace),
            );

            if edit_response.lost_focus() {
                // TextEdit loses focus on enter or tab. In any case, we'll apply the value if it is valid.
                match u16::from_str_radix(text.as_str(), 16) {
                    Ok(val) if val == 0 || val == 1 => {
                        log::debug!("Flag {} updated to {}", label, val);
                        *text = format!("{:X}", val);
                        //events.send(GuiEvent::Register16Update(reg, val));
                    }
                    _ => {
                        *text = "0".to_string(); // Reset to 0 if invalid
                    }
                }
                *updated = true;
            }

            ui.centered_and_justified(|ui| {
                if ui
                    .add(
                        egui::Label::new(egui::RichText::new(label).text_style(egui::TextStyle::Monospace))
                            .selectable(false),
                    )
                    .hovered()
                {
                    show_tooltip(ui.ctx(), ui.layer_id(), egui::Id::new("flag_tooltip"), |ui| {
                        ui.horizontal(|ui| {
                            ui.label(tip);
                        });
                    });
                }
            });
        });
    }
}
