use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

use super::debug::DebugItem;

pub struct Slider<T> {
    name: String,
    value: T,
    range: RangeInclusive<T>,
    has_been_updated: bool,
}

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

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn callback_update(&mut self, f: impl FnOnce(&mut T))
    where
        T: std::fmt::Debug,
    {
        if self.has_been_updated {
            log::trace!("Updating value of {} to {:?}", self.name, self.value);
            f(&mut self.value);
            self.has_been_updated = false;
        }
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
