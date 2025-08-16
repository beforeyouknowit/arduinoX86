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
    enums::MountAddress,
    events::GuiEventQueue,
    structs::BinaryBlob,
    widgets::mount_address_widget::MountAddressWidget,
};

pub struct BinaryView {
    pub name: String,
    pub icon_size: f32,
    pub mount_addr: MountAddress,
    pub mount_str: String,
    pub dt: DataTableWidget,
}

impl Default for BinaryView {
    fn default() -> Self {
        Self {
            name: "Program".into(),
            icon_size: 18.0,
            mount_addr: MountAddress::CsIp,
            mount_str: "0".to_string(),
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_data(&mut self, data: &[u8]) {
        self.dt.set_data(data);
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        blob: &mut BinaryBlob,
        _c_ctx: &mut ClientContext,
        _events: &mut GuiEventQueue,
    ) {
        ui.vertical(|ui| {
            ui.add(MountAddressWidget::new(&mut self.mount_addr, &mut self.mount_str));
            blob.set_mount_address(self.mount_addr.clone());
            ui.separator();
            self.dt.show(ui);
        });
    }
}
