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
    controls::cycle_table::CycleTable,
    events::{GuiEvent, GuiEventQueue},
};
use arduinox86_client::{ProgramState, ServerFlags, ServerStatus};
use egui_notify::Toasts;
use std::time::Instant;

pub struct ClientWindow {
    icon_size: f32,
    use_sdram_backend: bool,
    debug_enabled: bool,
    last_status_time: Option<Instant>,
    last_cycle_ct: u64,
    last_program_state: ProgramState,
    server_status: ServerStatus,
    effective_mhz: f32,
    cycle_table: CycleTable,
}

impl Default for ClientWindow {
    fn default() -> Self {
        Self {
            icon_size: 24.0,
            use_sdram_backend: false,
            debug_enabled: false,
            last_status_time: None,
            last_cycle_ct: 0,
            last_program_state: ProgramState::default(),
            server_status: Default::default(),
            effective_mhz: 0.0,
            cycle_table: Default::default(),
        }
    }
}

impl ClientWindow {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn init(&mut self, c_ctx: &ClientContext) {
        // Initialize the window with the current flags from the client context
        self.sync_flags(c_ctx);
    }

    pub fn sync_flags(&mut self, c_ctx: &ClientContext) {
        let flags = c_ctx.cached_flags();

        self.use_sdram_backend = flags & ServerFlags::USE_SDRAM_BACKEND != 0;
        self.debug_enabled = flags & ServerFlags::ENABLE_DEBUG != 0;
    }

    pub fn set_server_status(&mut self, c_ctx: &mut ClientContext, server_status: ServerStatus) {
        let update_time = Instant::now();

        if let Some(last_update) = self.last_status_time {
            // Calculate the effective MHz based on the time since the last update
            let elapsed_secs = update_time.duration_since(last_update).as_secs_f32();
            if elapsed_secs > 0.0 {
                self.effective_mhz = (server_status.cycle_ct - self.last_cycle_ct) as f32 / elapsed_secs / 1_000_000.0;
            }
            else {
                self.effective_mhz = 0.0; // Avoid division by zero
            }
        }
        else {
            self.effective_mhz = 0.0; // First update, no previous time to compare
        }

        if server_status.state != self.last_program_state {
            log::debug!("Server state changed to: {:?}", server_status);
            self.change_state(c_ctx, server_status.state);
        }

        self.last_program_state = self.server_status.state;
        self.server_status = server_status;

        self.last_cycle_ct = self.server_status.cycle_ct;
        self.last_status_time = Some(update_time);
    }

    pub fn change_state(&mut self, c_ctx: &mut ClientContext, new_state: ProgramState) {
        match new_state {
            ProgramState::StoreDone => {
                // Get the cycle states from the server.
                if let Ok(cycles) = c_ctx.client.get_cycle_states() {
                    self.cycle_table.set_cycles(cycles);
                }
                else {
                    log::error!("Failed to retrieve cycles from server.");
                }
            }
            _ => {}
        }
    }

    pub fn show(
        &mut self,
        e_ctx: &egui::Context,
        c_ctx: &mut ClientContext,
        events: &mut GuiEventQueue,
        toasts: &mut Toasts,
    ) {
        egui::Window::new("Client Connection")
            .default_width(800.0)
            .default_height(600.0)
            .show(e_ctx, |ui| {
                ui.vertical(|ui| {
                    egui::MenuBar::new().ui(ui, |ui| {
                        ui.menu_button("Options", |ui| {
                            if ui.checkbox(&mut self.debug_enabled, "Enable Serial Debug").changed() {
                                match c_ctx.set_flag_state(ServerFlags::ENABLE_DEBUG, self.debug_enabled) {
                                    Ok(true) => {
                                        let toggle_str = "Serial Debug enabled!".to_string();
                                        log::debug!("{}", toggle_str);
                                        toasts.success(toggle_str);
                                    }
                                    Ok(false) => {
                                        let toggle_str = "Serial Debug disabled!".to_string();
                                        log::debug!("{}", toggle_str);
                                        toasts.success(toggle_str);
                                    }
                                    Err(e) => {
                                        let toggle_str = format!("Failed to set Serial Debug state: {}", e);
                                        log::error!("{}", toggle_str);
                                        toasts.error(toggle_str);
                                        self.sync_flags(c_ctx);
                                    }
                                }
                            }

                            if ui.checkbox(&mut self.use_sdram_backend, "Use SDRAM Backend").changed() {
                                match c_ctx.set_flag_state(ServerFlags::USE_SDRAM_BACKEND, self.use_sdram_backend) {
                                    Ok(true) => {
                                        let toggle_str = "SDRAM backend enabled!".to_string();
                                        log::debug!("{}", toggle_str);
                                        toasts.success(toggle_str);
                                    }
                                    Ok(false) => {
                                        let toggle_str = "SDRAM backend disabled!".to_string();
                                        log::debug!("{}", toggle_str);
                                        toasts.success(toggle_str);
                                    }
                                    Err(e) => {
                                        let toggle_str = format!("Failed to set SDRAM backend: {}", e);
                                        log::error!("{}", toggle_str);
                                        toasts.error(toggle_str);
                                        self.sync_flags(c_ctx);
                                    }
                                }
                            }
                        });
                    });

                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new(format!("{}", egui_phosphor::regular::BOX_ARROW_UP))
                                    .size(self.icon_size),
                            )
                            .on_hover_text("Load registers")
                            .clicked()
                        {
                            // Do disconnect
                            events.push(GuiEvent::LoadRegisters);
                        }

                        if ui
                            .button(
                                egui::RichText::new(format!("{}", egui_phosphor::fill::ERASER)).size(self.icon_size),
                            )
                            .on_hover_text("Erase Memory")
                            .clicked()
                        {
                            // Do disconnect
                            events.push(GuiEvent::EraseMemory);
                        }

                        ui.separator();

                        if ui
                            .button(
                                egui::RichText::new(format!("{}", egui_phosphor::fill::PLAY_PAUSE))
                                    .size(self.icon_size),
                            )
                            .on_hover_text("Step")
                            .clicked()
                        {
                            // Do step
                        }

                        if ui
                            .button(egui::RichText::new(format!("{}", egui_phosphor::fill::PLAY)).size(self.icon_size))
                            .on_hover_text("Run Autonomously")
                            .clicked()
                        {
                            events.push(GuiEvent::RunProgram);
                        }

                        if ui
                            .button(
                                egui::RichText::new(format!("{}", egui_phosphor::fill::PLAY_PAUSE))
                                    .size(self.icon_size),
                            )
                            .on_hover_text("Pause")
                            .clicked()
                        {
                            // Do pause
                        }

                        if ui
                            .button(egui::RichText::new(format!("{}", egui_phosphor::fill::STOP)).size(self.icon_size))
                            .on_hover_text("Stop")
                            .clicked()
                        {
                            // Do pause
                        }

                        if ui
                            .button(
                                egui::RichText::new(format!("{}", egui_phosphor::regular::PLUGS)).size(self.icon_size),
                            )
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
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Server state:");
                        ui.label(format!("{:?}", self.server_status.state));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Cycle count:");
                        ui.label(self.server_status.cycle_ct.to_string());
                        ui.separator();
                        ui.label(format!("Effective MHz: {:.2}", self.effective_mhz));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Address latch:");
                        ui.label(format!("{:08X}", self.server_status.address_latch));
                    });
                });

                self.cycle_table.show(ui);
            });
    }
}
