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
use crate::enums::MountAddress;
use egui::Widget;
use std::fmt::Display;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MountAddressType {
    CsIp,
    FlatAddress,
}

impl Display for MountAddressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MountAddressType::CsIp => write!(f, "CS:IP"),
            MountAddressType::FlatAddress => write!(f, "Flat Address"),
        }
    }
}

pub struct MountAddressWidget<'a> {
    pub selected_type: MountAddressType,
    pub addr: &'a mut MountAddress,
    pub addr_str: &'a mut String,
}

impl<'a> MountAddressWidget<'a> {
    pub fn new(addr: &'a mut MountAddress, string_storage: &'a mut String) -> Self {
        let (selected_type, _addr_str) = match addr {
            MountAddress::CsIp => (MountAddressType::CsIp, "0".to_string()),
            MountAddress::FlatAddress(addr) => (MountAddressType::FlatAddress, format!("{:08X}", addr)),
        };
        Self {
            selected_type,
            addr,
            addr_str: string_storage,
        }
    }
}

impl Widget for MountAddressWidget<'_> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut selected = self.selected_type;

        let inner = ui.horizontal(|ui| {
            ui.label("Mount at:");
            let combo_resp = egui::ComboBox::new("mount_address_type", "")
                .selected_text(format!("{}", selected))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected, MountAddressType::CsIp, "CS:IP");
                    ui.selectable_value(&mut selected, MountAddressType::FlatAddress, "Flat Address");
                })
                .response;

            if selected != self.selected_type {
                // Update the selected type and reset the address string if changed
                self.selected_type = selected;
                match selected {
                    MountAddressType::CsIp => *self.addr_str = "0".to_string(),
                    MountAddressType::FlatAddress => {
                        *self.addr_str = format!("{:08X}", self.addr.flat_address().unwrap_or(0))
                    }
                };
            }

            let mut last_resp = combo_resp;

            // If FlatAddress is selected, draw the text field
            if matches!(self.selected_type, MountAddressType::FlatAddress) {
                let mut valid_addr = true;
                let parsed_val = u32::from_str_radix(self.addr_str.trim(), 16)
                    .map_err(|_| {
                        valid_addr = false;
                    })
                    .ok();

                // Change text color to red if invalid
                let text_color = if valid_addr {
                    ui.visuals().widgets.inactive.text_color()
                }
                else {
                    egui::Color32::RED
                };

                let edit_resp = ui.add(
                    egui::TextEdit::singleline(self.addr_str)
                        .text_color(text_color)
                        .hint_text("Enter hex address"),
                );

                if edit_resp.lost_focus() {
                    let parsed_val = u32::from_str_radix(self.addr_str.trim(), 16)
                        .map_err(|_| {
                            valid_addr = false;
                        })
                        .ok();

                    if let Some(parsed_val) = parsed_val {
                        // Update the address if valid
                        *self.addr_str = format!("{:08X}", parsed_val);
                    }
                    else {
                        // Reset to zero if invalid
                        *self.addr_str = format!("{:08X}", 0);
                    }
                }

                last_resp = edit_resp;

                // Only update the stored address if valid
                if let Some(val) = parsed_val {
                    *self.addr = MountAddress::FlatAddress(val);
                }
            }
            else {
                // CS:IP variant selected
                *self.addr = MountAddress::CsIp;
            }

            last_resp
        });

        // Return the closure’s result if possible, otherwise the container’s
        inner.inner
    }
}
