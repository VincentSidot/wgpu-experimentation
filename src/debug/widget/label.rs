use std::{cell::RefCell, rc::Rc};

use super::debug::DebugItem;

pub struct Label<T, S> {
    format: S,
    reference: T,
}

impl<T, S> Label<T, S>
where
    S: Fn(&T) -> String,
{
    pub fn new(reference: T, format: S) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { format, reference }))
    }

    pub fn set(&mut self, reference: T) {
        self.reference = reference;
    }

    pub fn as_text(&self) -> String {
        (self.format)(&self.reference)
    }
}

impl<T, S> DebugItem for Label<T, S>
where
    S: Fn(&T) -> String,
{
    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.label(self.as_text());
    }
}
