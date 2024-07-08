use std::{cell::RefCell, rc::Rc};

use super::widget::debug::DebugItem;


const INFO_COLOR: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
const WARNING_COLOR: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0);
const ERROR_COLOR: egui::Color32 = egui::Color32::from_rgb(0xFF, 0, 0);

pub enum DebugMessage {
    Info(String),
    Warning(String),
    Error(String),
}

impl DebugMessage {

    pub fn info<T: ToString>(msg: T) -> Self {
        Self::Info(msg.to_string())
    }

    pub fn warning<T: ToString>(msg: T) -> Self {
        Self::Warning(msg.to_string())
    }

    pub fn error<T: ToString>(msg: T) -> Self {
        Self::Error(msg.to_string())
    }

    pub fn to_label(&self) -> egui::Label {
        match self {
            Self::Info(msg) => egui::Label::new(
                egui::RichText::new(msg)
                    .color(INFO_COLOR)
            ),
            Self::Warning(msg) => egui::Label::new(
                egui::RichText::new(msg)
                    .color(WARNING_COLOR)
            ),
            Self::Error(msg) => egui::Label::new(
                egui::RichText::new(msg)
                    .color(ERROR_COLOR)
            ),
        }

    }
}

pub struct Debug {
    debug_console: Vec<DebugMessage>,
    debug_items: Vec<Rc<RefCell<dyn DebugItem>>>,
}

impl Debug {

    pub fn init() -> Self {
        Self {
            debug_console: Vec::new(),
            debug_items: Vec::new(),
        }
    }

    pub fn add_debug_item(&mut self, item: Rc<RefCell<dyn DebugItem>>) {
        self.debug_items
            .push(
                item
            );
    }

    pub fn info(&mut self, msg: &str) {
        self.debug_console.push(DebugMessage::info(msg));
    }

    #[allow(dead_code)]
    pub fn warning(&mut self, msg: &str) {
        self.debug_console.push(DebugMessage::warning(msg));
    }

    #[allow(dead_code)]
    pub fn error(&mut self, msg: &str) {
        self.debug_console.push(DebugMessage::error(msg));
    }

    pub fn run_ui(&mut self, ui: &egui::Context) {
        egui::Window::new("Debug Window")
            // .vscroll(true)
            .default_open(true)
            .max_width(1000.0)
            .max_height(800.0)
            .default_width(800.0)
            .resizable(true)
            // .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .show(&ui, |ui| {               
                for item in self.debug_items.iter_mut() {
                    item.borrow_mut().draw(ui, &mut self.debug_console);
                }
        });

        if !self.debug_console.is_empty() {
            egui::Window::new("Debug Console")
                .default_open(true)
                .show(&ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Debug Console");
                        if ui.button("Clear").on_hover_text("Clear the debug console").clicked() {
                            self.debug_console.clear();
                        };
                    });
                    ui.separator();
                    ui.vertical(|ui| {
    
                        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                            for msg in self.debug_console.iter() {
                                ui.add(msg.to_label());
                            }
                        });
                    });
            });
        }
        
    }

}