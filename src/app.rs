use std::{sync::mpsc::{Receiver, self}, collections::HashMap};

use clipboard::{ClipboardContext, ClipboardProvider};
use egui::{plot::{Line, PlotPoints}, Color32, RichText, Stroke, color_picker::Alpha, Sense};
use egui_extras::{TableBuilder, Column};
use rand::{Rng, distributions::Alphanumeric};
use regex::Regex;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{Server, ThreadMessage};

#[derive(serde::Deserialize, serde::Serialize)]
struct Channel {
    values: Vec<f64>,
    color: Color32,
    show: bool
}

impl Channel {
    fn new(color: Color32) -> Self {
        Self {
            values: Vec::new(),
            color,
            show: true
        }
    }
}

#[derive(Default)]
struct AddChannelWindow {
    open: bool,
    color: Color32
}

impl AddChannelWindow {
    fn show(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, channels: &mut HashMap<String, Channel>) {
        egui::Window::new("Add channel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Color: ");
                    egui::color_picker::color_edit_button_srgba(ui, &mut self.color, Alpha::Opaque);
                });
                ui.horizontal(|ui| {
                    if ui.button("Ok").clicked() {
                        channels.insert(generate_channel_hash(), Channel::new(self.color));
                        self.open = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.open = false;
                    }
                });
            });
        });
    }
}

struct StartServerWindow {
    open: bool,
    ip: String,
    port: String,
    ip_regex: Regex,
}

impl StartServerWindow {
    fn new() -> Self {
        Self {
            open: false,
            ip: "127.0.0.1".to_owned(),
            port: "6969".to_owned(),
            ip_regex: Regex::new("(([0-9]|[1-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])\\.){3}([0-9]|[1-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])").unwrap(),
        }
    }
}

impl StartServerWindow {
    fn show<F>(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, callback: F) where F: FnOnce() {
        egui::Window::new("Start server").show(ctx, |ui| {
            ui.vertical(|ui| {

                // TODO: learn regex lmao
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.ip);
                    if !self.ip_regex.is_match(&self.ip) {
                        ui.label(RichText::new("Invalid IP").color(Color32::RED)); // <--- this doesn't quite work
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.port);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Ok").clicked() {
                        self.open = false;
                        callback();
                    }
                    if ui.button("Cancel").clicked() {
                        self.open = false;
                    }
                });
                
            });
        });
    }
}

#[derive(Debug)]
enum MessageType {
    Info,
    Warning,
    Error
}

struct LoggingTab {
    lines: Vec<(String, MessageType)>,
    errors: bool,
    warnings: bool,
    info: bool,
}

impl LoggingTab {
    fn new() -> Self {
        Self {
            lines: vec![("Server is not running!".to_owned(), MessageType::Warning)],
            errors: true,
            warnings: true,
            info: true,
        }
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize, EnumIter, Debug, PartialEq, Clone, Copy)]
enum Tab {
    #[default]
    Plot,
    Log,
    Terrain,
    Map
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct StormtrackerApp {
    channels: HashMap<String, Channel>,
    current_tab: Tab,
    #[serde(skip)]
    add_channel_popup: AddChannelWindow,
    
    #[serde(skip)]
    logging_tab: LoggingTab,
    #[serde(skip)]
    server: Option<Server>,
    #[serde(skip)]
    message_receiver: Option<Receiver<ThreadMessage>>,
    #[serde(skip)]
    start_server_popup: StartServerWindow,

    #[serde(skip)]
    clipboard_ctx: ClipboardContext,

    #[serde(skip)]
    add_value: f64,
    #[serde(skip)]
    current_channel_hash: String,
}

impl Default for StormtrackerApp {
    fn default() -> Self {
        Self {
            channels: HashMap::new(),
            add_channel_popup: AddChannelWindow::default(),
            current_tab: Tab::default(),
            logging_tab: LoggingTab::new(),
            add_value: 0.0,
            current_channel_hash: String::new(),
            server: None,
            message_receiver: None,
            start_server_popup: StartServerWindow::new(),
            clipboard_ctx: ClipboardProvider::new().unwrap(),
        }
    }
}

impl StormtrackerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load state
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn receive_messages(&mut self) {
        match &self.message_receiver {
            Some(r) => {
                while let Ok(message) = r.try_recv() {
                    match message {
                        ThreadMessage::Error(text) => self.logging_tab.lines.push((text, MessageType::Error)),
                        ThreadMessage::Warning(text) => self.logging_tab.lines.push((text, MessageType::Warning)),
                        ThreadMessage::Info(text) => self.logging_tab.lines.push((text, MessageType::Info)),
                        ThreadMessage::PlotPoint(value) => match self.channels.get_mut(&self.current_channel_hash) {
                            Some(v) => v.values.push(value),
                            None => {},
                        },
                        ThreadMessage::PlotOnLine(k, v) => match self.channels.get_mut(&k) {
                            Some(channel) => channel.values.push(v),
                            None => {},
                        }
                    }
                }
            },
            None => {},
        }
    }

    fn tab_plot(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::right("my_right_panel").show(ctx, |ui| {

            ui.add(egui::DragValue::new(&mut self.add_value));
            if ui.button("Add datapoint").clicked() {
                match self.channels.get_mut(&self.current_channel_hash) {
                    Some(v) => v.values.push(self.add_value),
                    None => {},
                }
            }

            if ui.button("Test values").clicked() {
                for (i, (_, v)) in self.channels.iter_mut().enumerate() {
                    let factor = i as f64;
                    for x in 0..256 {
                        v.values.push(x as f64 * factor);
                    }
                }
            }

            // Channel table
            egui::CollapsingHeader::new("Channels").show(ui, |ui| {
                // Controls
                
                if ui.button("\u{2795} Add channel").clicked() {
                    self.add_channel_popup.open = true;
                }

                ui.separator();

                // Table itself
                let mut deletion_queue: Vec<String> = Vec::new();

                let table = TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::remainder())
                    .min_scrolled_height(0.0);
                
                table.header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Color");
                    });
                    header.col(|ui| {
                        ui.strong("Index");
                    });
                    header.col(|ui| {
                        ui.strong("Hash");
                    });
                    header.col(|ui| {
                        ui.strong("Show");
                    });
                })
                .body(|mut body| {
                    for (index, (hash, channel)) in self.channels.iter_mut().enumerate() {
                        body.row(18.0, |mut row| {
                            // Color
                            row.col(|ui| {
                                ui.label(RichText::new("\u{23FA}").color(channel.color));
                            });

                            // Index
                            row.col(|ui| {
                                ui.label(index.to_string());
                            });

                            // Hash
                            row.col(|ui| {
                                if ui.add(egui::Label::new(hash).sense(Sense::click())).clicked() {
                                    self.clipboard_ctx.set_contents(hash.to_string()).unwrap();
                                    println!("test");
                                }
                            });

                            // Show?
                            row.col(|ui| {
                                ui.checkbox(&mut channel.show, "");
                            });

                            // Clear channel
                            row.col(|ui| {
                                if ui.button("C").clicked() {
                                    channel.values.clear();
                                }
                            });

                            // Remove channel
                            row.col(|ui| {
                                if ui.button("\u{2796}").clicked() {
                                    deletion_queue.push(hash.to_string());
                                }
                            });
                        });
                    }
                });

                // Delete queued elements
                for hash in deletion_queue {
                    self.channels.remove(&hash);
                }
            });
        });
        
        // Plot
        // This is fun
        egui::CentralPanel::default().show(ctx, |ui| {
            let lines: Vec<Line> = self.channels
                .iter()
                .filter(|(_, v)| v.show)
                .map(|(_, v)| {
                    Line::new(
                        v.values
                            .iter()
                            .enumerate()
                            .map(|(x, y)| [x as f64, *y])
                            .collect::<PlotPoints>()
                    ).stroke(Stroke::new(2.0, v.color))
                })
                .collect();

            egui::plot::Plot::new("plot_0").show(ui, |plot_ui| {
                for line in lines {
                    plot_ui.line(line)
                }
            });
        });

        if self.add_channel_popup.open {
            self.add_channel_popup.show(ctx, frame, &mut self.channels);
        }
    }

    fn tab_log(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("my_bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {

                if ui.button("Start Server").clicked() {
                    self.start_server_popup.open = true;
                }

                ui.checkbox(&mut self.logging_tab.info, "Info");
                ui.checkbox(&mut self.logging_tab.warnings, "Warnings");
                ui.checkbox(&mut self.logging_tab.errors, "Errors");
            });
        });

        // Coloured and selectable text -- kind of hacky
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                let mut lines: Vec<(&str, Color32)> = self.logging_tab.lines
                    .iter()
                    .filter(|(_text, level)| {
                        match level {
                            MessageType::Info => self.logging_tab.info,
                            MessageType::Warning => self.logging_tab.warnings,
                            MessageType::Error => self.logging_tab.errors,
                        }
                    })
                    .map(|(text, level)| {
                        (text.as_str(), match level {
                            MessageType::Info => Color32::WHITE,
                            MessageType::Warning => Color32::GOLD,
                            MessageType::Error => Color32::RED,
                        })
                    })
                    .collect();
                for (text, color) in &mut lines {
                    ui.add(egui::TextEdit::singleline(text).text_color(*color));
                }
            });
        });

        if self.start_server_popup.open {
            self.start_server_popup.show(ctx, frame, || {
                self.server = Some(Server::new());
                let (sender, receiver) = mpsc::channel();
                self.message_receiver = Some(receiver);
                self.server.as_mut().unwrap().start(sender);
            });
        }
    }

    fn tab_terrain(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO
    }

    fn tab_map(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO
    }
}

impl eframe::App for StormtrackerApp {
    // Save state
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        // Get messages from server thread (if it exists)
        self.receive_messages();

        egui::TopBottomPanel::top("my_top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for tab in Tab::iter() {
                    ui.selectable_value(&mut self.current_tab, tab, format!("{:?}", tab));
                }
            });
        });

        match self.current_tab {
            Tab::Plot => self.tab_plot(ctx, frame),
            Tab::Log => self.tab_log(ctx, frame),
            Tab::Terrain => self.tab_terrain(ctx, frame),
            Tab::Map => self.tab_map(ctx, frame),
        }
    }
}

fn generate_channel_hash() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}
