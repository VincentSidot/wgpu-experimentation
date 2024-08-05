use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

use super::debug::DebugItem;

pub struct Slider<T> {
    name: String,
    value: T,
    range: RangeInclusive<T>,
    has_been_updated: bool,
}

#[allow(dead_code)]
impl<T> Slider<T> {
    pub fn new<S: ToString>(
        value: T,
        range: RangeInclusive<T>,
        name: S,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            name: name.to_string(),
            value,
            range,
            has_been_updated: false,
        }))
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.has_been_updated = true;
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn has_been_updated(&self) -> bool {
        self.has_been_updated
    }

    pub fn reset_updated(&mut self) {
        self.has_been_updated = false;
    }
}

impl<T> DebugItem for Slider<T>
where
    T: egui::emath::Numeric,
{
    fn draw(&mut self, ui: &mut egui::Ui) {
        // Draw the slider
        if ui
            .add(
                egui::Slider::new(&mut self.value, self.range.clone())
                    .text(&self.name),
            )
            .changed()
        {
            self.has_been_updated = true;
        };
    }
}
