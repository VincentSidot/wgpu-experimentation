use std::{cell::RefCell, rc::Rc};

use super::widget::{debug::DebugItem, Logger};

pub struct Debug {
    debug_widget: Logger,
    debug_items: Vec<Rc<RefCell<dyn DebugItem>>>,
}

struct Separator;
impl Separator {
    const fn new() -> Self {
        Self
    }
}
impl DebugItem for Separator {
    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.separator();
    }
}

impl Debug {
    pub fn init() -> Self {
        Self {
            debug_widget: Logger::new(),
            debug_items: Vec::new(),
        }
    }

    pub fn add_separator(&mut self) -> &mut Self {
        self.debug_items
            .push(Rc::new(RefCell::new(Separator::new())));
        self
    }

    pub fn add_debug_item(
        &mut self,
        item: Rc<RefCell<dyn DebugItem>>,
    ) -> &mut Self {
        self.debug_items.push(item);
        self
    }

    pub fn run_ui(&mut self, ui: &egui::Context) {
        egui::Window::new("Debug Window")
            // .vscroll(true)
            .default_open(false)
            .max_width(1000.0)
            .max_height(800.0)
            .default_width(800.0)
            .resizable(true)
            // .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .show(ui, |ui| {
                for item in self.debug_items.iter_mut() {
                    item.borrow_mut().draw(ui);
                }
            });

        if !self.debug_widget.is_empty() {
            egui::Window::new("Debug Console")
                .default_open(false)
                .max_width(Logger::MAX_WIDTH)
                .resizable(true)
                .show(ui, |ui| {
                    self.debug_widget.draw(ui);
                });
        }
    }
}
