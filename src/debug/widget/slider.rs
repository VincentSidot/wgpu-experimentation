use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

use super::debug::DebugItem;

pub struct Slider<T> {
    name: String,
    value: T,
    range: RangeInclusive<T>,
}

#[allow(dead_code)]
impl<T> Slider<T> {
    pub fn new<S: ToString>(value: T, range: RangeInclusive<T>, name: S) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            name: name.to_string(),
            value,
            range,
        }))
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T> DebugItem for Slider<T>
where
    T: egui::emath::Numeric,
{
    fn draw(&mut self, ui: &mut egui::Ui) {
        // Draw the slider
        ui.add(egui::Slider::new(&mut self.value, self.range.clone()).text(&self.name));
    }
}
