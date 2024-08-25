use std::{cell::RefCell, rc::Rc};

use super::debug::DebugItem;

pub struct Label<T> {
    format: fn(&T) -> String,
    reference: T,
}

impl<T> Label<T> {
    pub fn new(reference: T, format: fn(&T) -> String) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { format, reference }))
    }

    pub fn set(&mut self, reference: T) {
        self.reference = reference;
    }

    pub fn get(&self) -> &T {
        &self.reference
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.reference
    }

    pub fn as_text(&self) -> String {
        (self.format)(&self.reference)
    }
}

impl<T> DebugItem for Label<T> {
    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.label(self.as_text());
    }
}
