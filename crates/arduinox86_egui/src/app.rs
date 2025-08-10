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
    enums::CpuStateType,
    events::{GuiEvent, GuiEventQueue},
    serial_manager::SerialManager,
    windows::{ClientWindow, CodeEditor, RegisterWindow},
};

use crate::config::ConfigFile;
use arduinox86_client::{RegisterSetType, RemoteCpuRegisters, ServerFlags};
use std::{fs, io::Cursor, path::PathBuf, time::Duration};

use crate::{
    enums::{BinaryBlobType, ClientControlState, MountAddress},
    events::FrontendThreadEvent,
    resource_manager::ResourceManager,
    structs::BinaryBlob,
    windows::MemoryViewer,
};
use clap::Parser;
use egui::{
    containers::menu::{MenuButton, MenuConfig},
    PopupCloseBehavior,
};
use egui_notify::Toasts;

pub const SHORT_NOTIFICATION_TIME: Option<Duration> = Some(Duration::from_secs(2));
pub const NORMAL_NOTIFICATION_TIME: Option<Duration> = Some(Duration::from_secs(5));
pub const LONG_NOTIFICATION_TIME: Option<Duration> = Some(Duration::from_secs(8));

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Path to the TOML config file
    #[arg(long, value_name = "FILE", default_value = "./cfg/arduinox86_gui.toml")]
    config_file: PathBuf,
}

#[derive(Default)]
pub struct TransientAppState {
    ctx_init: bool,
    config: ConfigFile,
    serial_manager: SerialManager,
    resource_manager: ResourceManager,

    selected_serial_port: usize,

    client_ctx: Option<ClientContext>,
    client_window: ClientWindow,
    initial_register_window: RegisterWindow,
    code_editor_window: CodeEditor,
    memory_viewer_window: MemoryViewer,

    event_queue: GuiEventQueue,
    error_msg:   Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct GuiState {
    #[serde(skip)]
    toasts: Toasts,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    pub(crate) gs: GuiState,
    #[serde(skip)]
    ts: TransientAppState,

    #[serde(skip)]
    pub(crate) thread_sender:   crossbeam_channel::Sender<FrontendThreadEvent>,
    #[serde(skip)]
    pub(crate) thread_receiver: crossbeam_channel::Receiver<FrontendThreadEvent>,
}

impl Default for App {
    fn default() -> Self {
        let (thread_sender, thread_receiver) = crossbeam_channel::unbounded();
        Self {
            gs: GuiState {
                toasts: Toasts::new().with_anchor(egui_notify::Anchor::BottomRight),
            },
            ts: TransientAppState {
                ..TransientAppState::default()
            },
            thread_sender,
            thread_receiver,
        }
    }
}

impl App {
    /// Initialize the egui context, for visuals, etc.
    /// Tried doing this in new() but it didn't take effect.
    pub fn ctx_init(&mut self, ctx: &egui::Context) {
        ctx.set_visuals(egui::Visuals::dark());
        self.ts.ctx_init = true;
    }

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let cli = Cli::parse();

        // Load the config file.
        let config_text = match fs::read_to_string(&cli.config_file) {
            Ok(text) => text,
            Err(e) => {
                log::error!("Failed to read config file {}: {}", cli.config_file.display(), e);
                // exit
                std::process::exit(1);
            }
        };

        let config: ConfigFile = match toml::from_str(&config_text) {
            Ok(cfg) => cfg,
            Err(e) => {
                log::error!("Failed to parse config file {}: {}", cli.config_file.display(), e);
                // exit
                std::process::exit(1);
            }
        };

        // Create directories if they don't exist.
        if let Err(e) = fs::create_dir_all(&config.assembly_output_path) {
            log::error!(
                "Failed to create data directory {}: {}",
                config.assembly_output_path.display(),
                e
            );
            // exit
            std::process::exit(1);
        }

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        App {
            gs: GuiState {
                toasts: Toasts::new().with_anchor(egui_notify::Anchor::BottomRight),
            },
            ts: TransientAppState {
                config,
                serial_manager: SerialManager::new(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.ts.ctx_init {
            self.ctx_init(ctx);
        }

        self.gs.toasts.show(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.horizontal(|ui| {
                        if let Some(c_ctx) = &mut self.ts.client_ctx {
                            if c_ctx.control_state() == ClientControlState::Setup {
                                if ui.button("Load Program Binary").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Binary Files", &["bin", "hex"])
                                        .pick_file()
                                    {
                                        match BinaryBlob::from_path(
                                            "Program",
                                            MountAddress::CsIp,
                                            BinaryBlobType::Program,
                                            &path,
                                        ) {
                                            Ok(blob) => {
                                                if let Err(e) = self.ts.resource_manager.add_blob(blob) {
                                                    log::error!("Failed to load program binary: {}", e);
                                                    self.ts.error_msg =
                                                        Some(format!("Failed to load program binary: {}", e));
                                                }
                                                else {
                                                    self.gs
                                                        .toasts
                                                        .info("Program binary loaded successfully!")
                                                        .duration(NORMAL_NOTIFICATION_TIME);
                                                    log::info!("Program binary loaded successfully.");
                                                }
                                            }
                                            Err(e) => {
                                                log::error!("Failed to create program blob: {}", e);
                                                self.ts.error_msg = Some(format!("Failed to load program: {}", e));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });

                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                MenuButton::new("Serial Port")
                    .config(MenuConfig::default().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
                    .ui(ui, |ui| {
                        if ui.button("Refresh").clicked() {
                            self.ts.serial_manager.refresh();
                        }
                        ui.separator();
                        for (i, port) in self.ts.serial_manager.port_display_names().iter().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.radio_value(&mut self.ts.selected_serial_port, i, port).clicked() {
                                    log::debug!("Selected port: {}", port);
                                }
                            });
                        }
                    });
                ui.add_space(16.0);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("ArduinoX86 GUI ");
            ui.separator();

            let have_port = self
                .ts
                .serial_manager
                .port_names()
                .get(self.ts.selected_serial_port)
                .is_some();

            if have_port && self.ts.client_ctx.is_none() {
                if ui
                    .button(
                        egui::RichText::new(format!(
                            "⮉ Connect to ArduinoX86 Server on {}",
                            self.ts
                                .serial_manager
                                .port_names()
                                .get(self.ts.selected_serial_port)
                                .unwrap_or(&"No Port Selected".to_string())
                        ))
                        .size(18.0),
                    )
                    .clicked()
                {
                    // Do clicky stuff
                    match ClientContext::new(self.ts.selected_serial_port, &mut self.ts.serial_manager) {
                        Ok(client_ctx) => {
                            self.ts.error_msg = None;
                            self.ts.client_ctx = Some(client_ctx);
                            log::debug!(
                                "Connected to ArduinoX86 server on port: {}",
                                self.ts.selected_serial_port
                            );
                        }
                        Err(e) => {
                            log::error!("Failed to connect to ArduinoX86 server: {}", e);
                            self.ts.client_ctx = None;
                        }
                    }
                }
            }
            else if self.ts.client_ctx.is_some() {
                ui.label("Connected!");
            }
            else {
                ui.label("Select a serial port to connect.");
            }

            if let Some(err_msg) = &self.ts.error_msg {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err_msg));
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });

        // Render floating windows.
        if let Some(client_ctx) = &mut self.ts.client_ctx {
            self.ts
                .client_window
                .show(ctx, client_ctx, &mut self.ts.event_queue, &mut self.gs.toasts);
            let mut update_state = false;
            // Get the initial state from the client.
            let mut initial_state = client_ctx.initial_state_mut();
            let mut new_state = initial_state.clone();

            if !self.ts.initial_register_window.open() {
                self.ts.initial_register_window.set_regs(&initial_state.regs);
                *self.ts.initial_register_window.open_mut() = true;
            }

            if !self.ts.code_editor_window.open() {
                *self.ts.code_editor_window.open_mut() = true;
            }

            match &initial_state.regs {
                RemoteCpuRegisters::V3(regs) => {
                    self.ts.initial_register_window.show(
                        ctx,
                        CpuStateType::Initial,
                        RegisterSetType::Intel386,
                        &mut self.ts.event_queue,
                    );

                    let new_regs = self
                        .ts
                        .initial_register_window
                        .regs(RegisterSetType::from(&initial_state.regs));

                    new_state.regs = new_regs;
                    update_state = true;
                }
                _ => {
                    log::warn!(
                        "Unsupported register type: {}",
                        RegisterSetType::from(&initial_state.regs)
                    );
                    self.ts.error_msg = Some("Unsupported register type".to_string());
                }
            }

            self.ts.code_editor_window.show(ctx);
            self.ts.resource_manager.show(ctx, client_ctx, &mut self.ts.event_queue);
            self.ts
                .memory_viewer_window
                .show(ctx, client_ctx, &mut self.ts.event_queue);

            if update_state {
                client_ctx.set_initial_state(&new_state);
            }
        }

        // Handle events.
        self.handle_events(ctx);
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

impl App {
    /// Handle events from the GUI event queue.
    fn handle_events(&mut self, _c_ctx: &egui::Context) {
        if let Some(client_ctx) = &mut self.ts.client_ctx {
            while let Some(event) = self.ts.event_queue.pop() {
                match event {
                    GuiEvent::LoadRegisters => {
                        let initial_state = client_ctx.initial_state();

                        let mut buf = Cursor::new(Vec::<u8>::new());

                        initial_state.regs.write(&mut buf).unwrap_or_else(|e| {
                            log::error!("Failed to write registers: {}", e);
                        });

                        if let Err(e) = client_ctx
                            .client
                            .load_registers_from_buf(RegisterSetType::from(&initial_state.regs), buf.get_ref())
                        {
                            log::error!("Failed to load registers: {}", e);
                            self.ts.error_msg = Some(format!("Failed to load registers: {}", e));
                        }
                        else {
                            log::debug!("Registers loaded successfully.");
                        }
                    }
                    GuiEvent::ReadMemory { address, size } => match client_ctx.read_memory(address, size) {
                        Ok(data) => {
                            self.ts.memory_viewer_window.set_data(data);
                            log::debug!(
                                "Memory read successfully from address {:#x} with size {}",
                                address,
                                size
                            );
                        }
                        Err(e) => {
                            log::error!("Failed to read memory: {}", e);
                            self.ts.error_msg = Some(format!("Failed to read memory: {}", e));
                        }
                    },
                    GuiEvent::RunProgram => {
                        // Load the binary resources into memory.
                        for blob in self.ts.resource_manager.blobs() {
                            let resolved_mount_address = match blob.mount_address {
                                MountAddress::FixedAddress(addr) => addr,
                                MountAddress::CsIp => self
                                    .ts
                                    .initial_register_window
                                    .regs(RegisterSetType::Intel386)
                                    .code_address(),
                            };

                            if let Err(e) = client_ctx.client.set_memory(resolved_mount_address, &blob.data) {
                                self.gs
                                    .toasts
                                    .error(format!("Failed to load binary blob: {}", e))
                                    .duration(LONG_NOTIFICATION_TIME);

                                log::error!("Failed to load binary blob: {}", e);
                                self.ts.error_msg = Some(format!("Failed to load binary blob: {}", e));
                                return;
                            }
                            else {
                                self.gs
                                    .toasts
                                    .success(format!("Binary blob: {} loaded successfully!", blob.name))
                                    .duration(NORMAL_NOTIFICATION_TIME);
                                log::debug!(
                                    "Binary blob: {} loaded successfully at address {:#x}",
                                    blob.name,
                                    resolved_mount_address
                                );
                            }
                        }

                        match client_ctx.set_flag_state(ServerFlags::EXECUTE_AUTOMATIC, true) {
                            Ok(_) => {
                                self.gs
                                    .toasts
                                    .success("Automatic execution flag set.")
                                    .duration(NORMAL_NOTIFICATION_TIME);
                                log::debug!("Automatic execution flag set.");
                            }
                            Err(e) => {
                                self.gs
                                    .toasts
                                    .error(format!("Failed to set automatic execution flag: {}", e))
                                    .duration(LONG_NOTIFICATION_TIME);
                                log::error!("Failed to set automatic execution flag: {}", e);
                            }
                        }

                        // Load registers.
                        let initial_state = client_ctx.initial_state();

                        let mut buf = Cursor::new(Vec::<u8>::new());
                        initial_state
                            .regs
                            .write(&mut buf)
                            .expect("Failed to write registers to buffer");

                        if let Err(e) = client_ctx
                            .client
                            .load_registers_from_buf(RegisterSetType::from(&initial_state.regs), buf.get_ref())
                        {
                            self.gs
                                .toasts
                                .error(format!("Failed to load registers: {}", e))
                                .duration(LONG_NOTIFICATION_TIME);
                            log::error!("Failed to load registers: {}", e);
                            self.ts.error_msg = Some(format!("Failed to load registers: {}", e));
                        }
                        else {
                            self.gs
                                .toasts
                                .success("Registers loaded successfully!")
                                .duration(NORMAL_NOTIFICATION_TIME);
                            log::debug!("Registers loaded successfully.");
                        }
                    }
                }
            }
        }
    }
}
