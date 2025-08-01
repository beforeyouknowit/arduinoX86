use crate::serial_manager::SerialManager;

#[derive(Default)]
pub struct TransientAppState {
    ctx_init: bool,
    serial_manager: SerialManager,

    selected_serial_port: usize,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ArduinoX86 {
    // Example stuff:
    label: String,

    #[serde(skip)]
    ts: TransientAppState,
}

impl Default for ArduinoX86 {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            ts:    TransientAppState::default(),
        }
    }
}

impl ArduinoX86 {
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

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        ArduinoX86 {
            ts: TransientAppState {
                serial_manager: SerialManager::new(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl eframe::App for ArduinoX86 {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.ts.ctx_init {
            self.ctx_init(ctx);
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });

                    ui.menu_button("Serial Port", |ui| {
                        if ui.button("Refresh").clicked() {
                            self.ts.serial_manager.refresh();
                        }
                        ui.separator();
                        for (i, port) in self.ts.serial_manager.port_names().iter().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.radio_value(&mut self.ts.selected_serial_port, i, port).clicked() {
                                    // Here you would handle the port selection, e.g., connect to it.
                                    println!("Selected port: {}", port);
                                }
                            });
                        }
                    });
                    ui.add_space(16.0);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("ArduinoX86 GUI ");
            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
