use std::{
    path::{Path, PathBuf},
    collections::HashMap,
};
use eframe::egui::{self, ScrollArea, ProgressBar, Ui};
use rfd::FileDialog;
use crate::{
    file_scanner::{Scanner, ScannerMessage},
    file_utils::{create_hardlink, move_file},
    preview::Preview,
};

pub struct DuplicateFinderApp {
    directory: String,
    filters: Vec<String>,
    new_filter: String,
    duplicates: HashMap<Vec<u8>, Vec<PathBuf>>,
    scanner: Scanner,
    scanning: bool,
    progress: f32,
    selected_file: Option<PathBuf>,
    preview: Option<Preview>,
    error: Option<String>,
}

impl DuplicateFinderApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            directory: String::new(),
            filters: Vec::new(),
            new_filter: String::new(),
            duplicates: HashMap::new(),
            scanner: Scanner::new(),
            scanning: false,
            progress: 0.0,
            selected_file: None,
            preview: None,
            error: None,
        }
    }

    fn handle_scanner_messages(&mut self) {
        while let Ok(message) = self.scanner.receiver().try_recv() {
            match message {
                ScannerMessage::Progress(progress) => {
                    self.progress = progress;
                }
                ScannerMessage::Found(duplicates) => {
                    self.duplicates = duplicates;
                    self.scanning = false;
                }
                ScannerMessage::Error(error) => {
                    self.error = Some(error);
                    self.scanning = false;
                }
            }
        }
    }

    fn show_directory_section(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("📁 Select Directory").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    self.directory = path.display().to_string();
                }
            }
            ui.text_edit_singleline(&mut self.directory);
        });

        if !self.directory.is_empty() && !self.scanning {
            if ui.button("🔍 Start Scan").clicked() {
                self.start_scan();
            }
        }

        if self.scanning {
            ui.add(ProgressBar::new(self.progress).text("Scanning..."));
        }
    }

    fn show_filters_section(&mut self, ui: &mut Ui) {
        ui.collapsing("🔧 Filters", |ui| {
            ui.horizontal(|ui| {
                ui.label("Exclude extension:");
                if ui.text_edit_singleline(&mut self.new_filter).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !self.new_filter.is_empty() {
                        self.filters.push(self.new_filter.clone());
                        self.new_filter.clear();
                    }
                }
            });

            ui.horizontal_wrapped(|ui| {
                let mut to_remove = None;
                for (idx, filter) in self.filters.iter().enumerate() {
                    ui.label(format!(".{}", filter));
                    if ui.small_button("❌").clicked() {
                        to_remove = Some(idx);
                    }
                }
                if let Some(idx) = to_remove {
                    self.filters.remove(idx);
                }
            });
        });
    }

    fn show_duplicates_section(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            for (_, paths) in &self.duplicates {
                if let Some(original) = paths.first() {
                    ui.collapsing(format!("📄 {}", original.display()), |ui| {
                        for path in paths {
                            ui.horizontal(|ui| {
                                if ui.selectable_label(
                                    self.selected_file.as_ref() == Some(path),
                                    path.display().to_string()
                                ).clicked() {
                                    self.selected_file = Some(path.clone());
                                    if let Ok(preview) = Preview::from_file(path) {
                                        self.preview = Some(preview);
                                    }
                                }

                                if path != original {
                                    if ui.button("🗑️ Delete").clicked() {
                                        if let Err(e) = std::fs::remove_file(path) {
                                            self.error = Some(e.to_string());
                                        }
                                    }
                                    if ui.button("🔗 Hardlink").clicked() {
                                        if let Err(e) = create_hardlink(original, path) {
                                            self.error = Some(e.to_string());
                                        }
                                    }
                                    if ui.button("📦 Move").clicked() {
                                        if let Some(dst) = FileDialog::new()
                                            .set_file_name(path.file_name().unwrap().to_str().unwrap())
                                            .save_file() {
                                            if let Err(e) = move_file(path, &dst) {
                                                self.error = Some(e.to_string());
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    });
                }
            }
        });
    }

    fn show_preview_section(&mut self, ui: &mut Ui) {
        if let Some(preview) = &self.preview {
            ui.collapsing("👁️ Preview", |ui| {
                match preview {
                    Preview::Text(content) => {
                        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            ui.text_edit_multiline(&mut content.as_str());
                        });
                    }
                    Preview::Image(_bytes, format) => {
                        ui.label(format!("Image preview ({:?})", format));
                        // Here we could add actual image rendering using egui's image support
                    }
                    Preview::Binary => {
                        ui.label("Binary file (no preview available)");
                    }
                }
            });
        }
    }

    fn start_scan(&mut self) {
        self.scanning = true;
        self.progress = 0.0;
        self.duplicates.clear();
        self.error = None;
        self.selected_file = None;
        self.preview = None;
        
        self.scanner.start_scan(
            Path::new(&self.directory),
            self.filters.clone(),
        );
    }
}

impl eframe::App for DuplicateFinderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_scanner_messages();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("DupFi - Duplicate File Finder");

            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, error);
            }

            self.show_directory_section(ui);
            self.show_filters_section(ui);
            
            ui.separator();

            if !self.duplicates.is_empty() {
                self.show_duplicates_section(ui);
                self.show_preview_section(ui);
            }
        });

        // Request repaint while scanning to update progress
        if self.scanning {
            ctx.request_repaint();
        }
    }
}
