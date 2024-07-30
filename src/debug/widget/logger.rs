use core::f32;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    ptr::{addr_of, addr_of_mut},
    rc::Rc,
    sync::Arc,
    usize,
};

use egui::{text, FontId};
use regex::Regex;

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

impl LoggerMessage {
    fn color(&self) -> egui::Color32 {
        match self.level {
            log::Level::Info => INFO_COLOR,
            log::Level::Warn => WARNING_COLOR,
            log::Level::Error => ERROR_COLOR,
            log::Level::Debug => DEBUG_COLOR,
            log::Level::Trace => TRACE_COLOR,
        }
    }

    fn should_display(&self, show: &[bool; 5]) -> bool {
        match self.level {
            log::Level::Info => show[0],
            log::Level::Warn => show[1],
            log::Level::Error => show[2],
            log::Level::Debug => show[3],
            log::Level::Trace => show[4],
        }
    }

    fn level(&self) -> &str {
        match self.level {
            log::Level::Info => "INFO",
            log::Level::Warn => "WARN",
            log::Level::Error => "ERRO",
            log::Level::Debug => "DEBU",
            log::Level::Trace => "TRAC",
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
}

impl Logger {
    pub const MAX_WIDTH: f32 = 800.0;

    pub fn new() -> Self {
        Self {
            show: [true; 5],
            filter: String::new(),
            sensitive: false,
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
        let mut displayed_count = 0;
        let log_count = unsafe { LOGGER.items.len() };

        ui.horizontal(|ui| {
            if ui
                .button("Clear")
                .on_hover_text("Clear the debug console")
                .clicked()
            {
                unsafe {
                    LOGGER.items.clear();
                }
            }
            ui.checkbox(&mut self.show[0], "Info");
            ui.checkbox(&mut self.show[1], "Warning");
            ui.checkbox(&mut self.show[2], "Error");
            ui.checkbox(&mut self.show[3], "Debug");
            ui.checkbox(&mut self.show[4], "Trace");
        });
        ui.separator();

        let computed_filter = if self.sensitive {
            self.filter.clone()
        } else {
            format!("(?i){}", self.filter)
        };

        ui.vertical(|ui| {
            for (index, item) in unsafe { LOGGER.items.iter() }.enumerate() {
                if item.should_display(&self.show) {
                    ui.horizontal(|ui| {
                        ui.label(format!("{:03}", index + 1));
                        ui.add(egui::Label::new(
                            egui::RichText::new(item.level()).color(item.color()),
                        ))
                        .on_hover_text(format!("{} -> {}:{}", item.source, item.file, item.line));
                        ui.add({
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

                            if self.filter.is_empty() {
                                egui::RichText::new(item.content.as_str()).append_to(
                                    &mut layout_job,
                                    &style,
                                    egui::FontSelection::Default,
                                    egui::Align::Center,
                                );
                            } else {
                                let filter = Regex::new(&computed_filter)
                                    .unwrap_or_else(|_| Regex::new(".*").unwrap());
                                let mut last_after = 0;
                                for matched in filter.find_iter(&item.content) {
                                    displayed_count += 1;
                                    let before_match = &item.content[last_after..matched.start()];
                                    let current_match =
                                        &item.content[matched.start()..matched.end()];
                                    last_after = matched.end();

                                    egui::RichText::new(before_match).append_to(
                                        &mut layout_job,
                                        &style,
                                        egui::FontSelection::Default,
                                        egui::Align::Center,
                                    );

                                    egui::RichText::new(current_match)
                                        .background_color(egui::Color32::YELLOW)
                                        .append_to(
                                            &mut layout_job,
                                            &style,
                                            egui::FontSelection::Default,
                                            egui::Align::Center,
                                        );
                                }

                                let after_match = &item.content[last_after..];
                                egui::RichText::new(after_match).append_to(
                                    &mut layout_job,
                                    &style,
                                    egui::FontSelection::Default,
                                    egui::Align::Center,
                                );
                            }

                            egui::Label::new(layout_job).wrap(true)
                        });
                    });
                }
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui
                .button("Clear")
                .on_hover_text("Clear the filter")
                .clicked()
            {
                self.filter.clear();
            }
            ui.checkbox(&mut self.sensitive, "Case sensitive");
            ui.separator();
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .hint_text("Filter")
                    .desired_width(200.0),
            );
            ui.label(format!("{}/{}", displayed_count, log_count));
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
