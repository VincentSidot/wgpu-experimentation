use core::f32;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    collections::VecDeque,
    ptr::{addr_of, addr_of_mut},
    rc::Rc,
    sync::Arc,
    usize,
};

use egui::{
    text::{self, LayoutJob},
    FontId, Label, RichText,
};
use log::Level;

use super::debug::DebugItem;

const INFO_COLOR: egui::Color32 = egui::Color32::WHITE;
const WARNING_COLOR: egui::Color32 = egui::Color32::YELLOW;
const ERROR_COLOR: egui::Color32 = egui::Color32::RED;
const DEBUG_COLOR: egui::Color32 = egui::Color32::from_rgb(0x80, 0x00, 0x80);
const TRACE_COLOR: egui::Color32 = egui::Color32::BLUE;

struct LoggerMessage {
    content: String,
    source: String,
    level: log::Level,
    file: String,
    line: u32,
}

#[derive(Clone)]
enum TextSlice {
    Normal(String),
    Highlighted(String),
}

#[derive(Clone)]
struct ComputedLoggerMessage {
    index_content: String,
    log_content: Vec<TextSlice>,
    hover_text: String,
    level: log::Level,
}

impl LoggerMessage {
    fn compute(
        &self,
        filter: Option<(&str, bool)>,
        index: usize,
    ) -> Option<ComputedLoggerMessage> {
        let mut log_content = Vec::new();
        if let Some(filter) = filter {
            let sensitive = filter.1;

            let filter = filter.0;
            let filter_len = filter.len();

            let mut last_after = 0;

            let matched_iter: Vec<usize> = if sensitive {
                self.content.match_indices(filter).map(|x| x.0).collect()
            } else {
                self.content
                    .to_lowercase()
                    .match_indices(filter.to_lowercase().as_str())
                    .map(|x| x.0)
                    .collect()
            };

            for start in matched_iter {
                let end = start + filter_len;
                let before_match = &self.content[last_after..start];
                let current_match = &self.content[start..end];
                last_after = end;
                log_content.push(TextSlice::Normal(before_match.to_string()));
                log_content
                    .push(TextSlice::Highlighted(current_match.to_string()));
            }
            if last_after != 0 {
                let after_match = &self.content[last_after..];
                log_content.push(TextSlice::Normal(after_match.to_string()));
            } else {
                return None;
            }
        } else {
            log_content.push(TextSlice::Normal(self.content.clone()));
        };

        Some(ComputedLoggerMessage {
            index_content: format!("{:04}", index + 1),
            log_content,
            hover_text: format!(
                "{} -> {}:{}\n{}",
                self.source, self.file, self.line, self.content
            ),
            level: self.level,
        })
    }
}

fn level(level: &Level) -> &str {
    match level {
        log::Level::Info => "INFO",
        log::Level::Warn => "WARN",
        log::Level::Error => "ERRO",
        log::Level::Debug => "DEBU",
        log::Level::Trace => "TRAC",
    }
}

fn color(level: &Level) -> egui::Color32 {
    match level {
        log::Level::Info => INFO_COLOR,
        log::Level::Warn => WARNING_COLOR,
        log::Level::Error => ERROR_COLOR,
        log::Level::Debug => DEBUG_COLOR,
        log::Level::Trace => TRACE_COLOR,
    }
}

impl ComputedLoggerMessage {
    fn should_display(&self, show: &[bool; 5]) -> bool {
        match self.level {
            log::Level::Info => show[0],
            log::Level::Warn => show[1],
            log::Level::Error => show[2],
            log::Level::Debug => show[3],
            log::Level::Trace => show[4],
        }
    }
}

struct StaticLogger {
    items: Vec<LoggerMessage>,
    level: log::Level,
}

static mut LOGGER: StaticLogger = StaticLogger {
    items: Vec::new(),
    level: log::Level::Trace,
};

pub struct Logger {
    show: [bool; 5],
    filter: String,
    sensitive: bool,
    log: Vec<ComputedLoggerMessage>,
    last_index: usize,
    paused: bool,
}

impl Logger {
    pub const MAX_WIDTH: f32 = 300.0;

    pub fn new() -> Self {
        Self {
            show: [true; 5],
            filter: String::new(),
            sensitive: false,
            log: Vec::new(),
            last_index: 0,
            paused: false,
        }
    }

    pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            #[allow(static_mut_refs)]
            // SAFETY: We are the only ones accessing it
            log::set_logger(&LOGGER).map_err(|_| "Failed to set logger")?;
            log::set_max_level(log::LevelFilter::Trace);
        }
        log::info!("Logger initialized");
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        unsafe { LOGGER.items.is_empty() }
    }
}

impl DebugItem for Logger {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let log_count = unsafe { LOGGER.items.len() };
        let mut filtred_log = Vec::new();

        ui.horizontal(|ui| {
            if ui
                .button("Clear")
                .on_hover_text("Clear the debug console")
                .clicked()
            {
                unsafe {
                    LOGGER.items.clear();
                }
                self.last_index = 0;
            }
            ui.checkbox(&mut self.show[0], "Info");
            ui.checkbox(&mut self.show[1], "Warning");
            ui.checkbox(&mut self.show[2], "Error");
            ui.checkbox(&mut self.show[3], "Debug");
            ui.checkbox(&mut self.show[4], "Trace");
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.paused, "Pause");
            if ui
                .button("Clear")
                .on_hover_text("Clear the filter")
                .clicked()
            {
                self.filter.clear();
            }
            if ui.checkbox(&mut self.sensitive, "Case sensitive").clicked() {
                self.last_index = 0;
            }
            ui.separator();
            if ui
                .add(
                    egui::TextEdit::singleline(&mut self.filter)
                        .hint_text("Filter")
                        .desired_width(200.0),
                )
                .changed()
            {
                self.last_index = 0;
            }
            ui.separator();

            let start_index = self.last_index;

            let filter = if self.filter.is_empty() {
                None
            } else {
                Some((self.filter.as_str(), self.sensitive))
            };

            if self.last_index == 0 {
                let mut parsed_item = 0;
                self.log = unsafe { LOGGER.items.iter() }
                    .enumerate()
                    .filter_map(|(index, item)| {
                        parsed_item += 1;
                        item.compute(filter, index)
                    })
                    .collect();
                self.last_index = parsed_item;
            } else if !self.paused {
                let mut parsed_item = 0;
                self.log.extend(
                    unsafe { LOGGER.items[start_index..].iter() }
                        .enumerate()
                        .filter_map(|(index, item)| {
                            parsed_item += 1;
                            item.compute(filter, index + start_index)
                        }),
                );
                self.last_index += parsed_item;
            }
            // if self.last_index > 0 {
            //     self.last_index -= 1;
            // }

            filtred_log = self
                .log
                .iter_mut()
                .filter(|x| x.should_display(&self.show))
                .map(|x| x.clone())
                .rev()
                .collect::<Vec<_>>();

            ui.label(format!("{}/{}", self.log.len(), log_count));
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for message in filtred_log {
                ui.horizontal(|ui| {
                    let message_index = message.index_content;
                    let message_content = message.log_content;
                    let message_hover = message.hover_text;
                    let message_level = level(&message.level);
                    let message_color = color(&message.level);

                    ui.label(message_index);
                    ui.add(Label::new(
                        RichText::new(message_level).color(message_color),
                    ))
                    .on_hover_text(message_hover);

                    let style = egui::Style::default();
                    let mut layout_job = text::LayoutJob {
                        wrap: egui::text::TextWrapping {
                            max_width: Self::MAX_WIDTH,
                            max_rows: usize::MAX,
                            break_anywhere: false,
                            overflow_character: None,
                        },
                        justify: true,
                        ..Default::default()
                    };

                    for substring in message_content {
                        match substring {
                            TextSlice::Normal(content) => {
                                RichText::new(content).append_to(
                                    &mut layout_job,
                                    &style,
                                    egui::FontSelection::Default,
                                    egui::Align::Center,
                                );
                            }
                            TextSlice::Highlighted(content) => {
                                RichText::new(content)
                                    .background_color(egui::Color32::YELLOW)
                                    .append_to(
                                        &mut layout_job,
                                        &style,
                                        egui::FontSelection::Default,
                                        egui::Align::Center,
                                    );
                            }
                        }
                    }

                    ui.add(Label::new(layout_job).wrap(true));
                });
            }
        });
    }
}

impl log::Log for StaticLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= unsafe { LOGGER.level }
    }

    fn log(&self, record: &log::Record) {
        // Discard logs from wgpu
        if record.target().starts_with("wgpu") {
            return;
        }
        // Discard logs from naga
        if record.target().starts_with("naga") {
            return;
        }
        // Discard logs from egui
        if record.target().starts_with("egui") {
            return;
        }

        if self.enabled(record.metadata()) {
            unsafe {
                LOGGER.items.push(LoggerMessage {
                    content: record.args().to_string(),
                    source: record.target().to_string(),
                    level: record.level(),
                    file: record.file().unwrap_or("unknown").to_string(),
                    line: record.line().unwrap_or(0),
                });
            }
        }
    }

    fn flush(&self) {}
}
