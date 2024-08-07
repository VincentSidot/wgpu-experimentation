use std::{cell::RefCell, rc::Rc};

use super::debug::DebugItem;

pub struct Button {
    label: String,
    has_been_pressed: bool,
}

impl Button {
    pub fn new<S: ToString>(label: S) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            label: label.to_string(),
            has_been_pressed: false,
        }))
    }

    pub fn callback_update(&mut self, f: impl FnOnce()) {
        if self.has_been_pressed {
            f();
            self.has_been_pressed = false;
        }
    }
}

impl DebugItem for Button {
    fn draw(&mut self, ui: &mut egui::Ui) {
        if ui.button(&self.label).clicked() {
            self.has_been_pressed = true;
        }
    }
}
