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

use std::collections::HashMap;

use crate::windows::{BinaryView, CodeEditor};

use crate::{client::ClientContext, events::GuiEventQueue, resource_manager::ResourceManager};
use anyhow::{bail, Result};
use egui_extras::syntax_highlighting::SyntectSettings;

pub enum Window {
    BinaryView { open: bool, window: BinaryView },
    CodeEditor { open: bool, window: CodeEditor },
}

#[derive(Default)]
pub struct WindowManager {
    pub blob_windows: HashMap<String, Window>,
    pub code_windows: HashMap<String, Window>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            blob_windows: HashMap::new(),
            code_windows: HashMap::new(),
        }
    }

    pub fn add_blob(&mut self, name: String, window: BinaryView) {
        self.blob_windows
            .insert(name, Window::BinaryView { open: true, window });
    }

    pub fn rename_blob(&mut self, old_name: &str, new_name: String) -> Result<()> {
        if let Some(window) = self.blob_windows.remove(old_name) {
            self.blob_windows.insert(new_name, window);
            Ok(())
        }
        else {
            bail!("Blob window '{}' does not exist.", old_name);
        }
    }

    pub fn add_code(&mut self, name: String, window: CodeEditor) {
        self.code_windows
            .insert(name, Window::CodeEditor { open: true, window });
    }

    pub fn rename_code(&mut self, old_name: &str, new_name: String) -> Result<()> {
        if let Some(window) = self.code_windows.remove(old_name) {
            self.code_windows.insert(new_name, window);
            Ok(())
        }
        else {
            bail!("Code editor '{}' does not exist.", old_name);
        }
    }

    pub fn show(
        &mut self,
        e_ctx: &egui::Context,
        c_ctx: &mut ClientContext,
        rm: &mut ResourceManager,
        syntect_settings: &SyntectSettings,
        events: &mut GuiEventQueue,
    ) {
        let mut close_windows = Vec::new();
        for (name, window) in &mut self.blob_windows {
            if let Window::BinaryView { open, window } = window {
                if let Some(blob) = rm.blob_mut(name) {
                    let initial_open = *open;
                    egui::Window::new(format!("Binary View: {}", name))
                        .open(open)
                        .default_width(800.0)
                        .default_height(600.0)
                        .show(e_ctx, |ui| {
                            window.show(ui, blob, c_ctx, events);
                        });

                    if initial_open && !*open {
                        // Window was closed
                        close_windows.push(name.clone());
                    }
                }
                else {
                    log::error!("Failed to find blob data for window: {}", name);
                }
            }
        }

        // Close closed windows
        for name in close_windows {
            log::debug!("Closing binary view window: {}", name);
            _ = rm.remove_blob(&name);
            self.blob_windows.remove(&name);
        }

        for (_name, editor) in &mut self.code_windows {
            if let Window::CodeEditor { window, .. } = editor {
                window.show(e_ctx, syntect_settings, events);
            }
        }
    }

    pub fn new_code_window(&mut self, base_name: &str, code: Option<String>) {
        let name = self.available_code_name(base_name);
        let mut editor = CodeEditor::new(&name);

        if let Some(code) = code {
            editor.set_code(code);
        }
        log::debug!("new_code_window(): Creating new code window: {}", name);
        *editor.open_mut() = true;
        self.code_windows.insert(
            name,
            Window::CodeEditor {
                open:   true,
                window: editor,
            },
        );
    }

    fn available_code_name(&self, base: &str) -> String {
        let mut name = base.to_string();
        let mut counter = 1;

        while self.code_windows.contains_key(&name) {
            name = format!("{}{}", base, counter);
            counter += 1;
        }

        name
    }

    pub fn code_window(&self, name: &str) -> Option<&CodeEditor> {
        self.code_windows.get(name).map(|w| {
            if let Window::CodeEditor { window, .. } = w {
                window
            }
            else {
                panic!("Expected CodeEditor window type");
            }
        })
    }

    pub fn code_window_mut(&mut self, name: &str) -> Option<&mut CodeEditor> {
        self.code_windows.get_mut(name).map(|w| {
            if let Window::CodeEditor { window, .. } = w {
                window
            }
            else {
                panic!("Expected CodeEditor window type");
            }
        })
    }
}
