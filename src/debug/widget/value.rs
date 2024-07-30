use std::{cell::RefCell, rc::Rc};

use super::debug::DebugItem;

pub struct Value<T> {
    name: String,
    reference: T,
}

#[allow(dead_code)]
impl<T> Value<T> {
    pub fn new<S: ToString>(reference: T, name: S) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            name: name.to_string(),
            reference,
        }))
    }

    pub fn get(&self) -> &T {
        &self.reference
    }

    pub fn set(&mut self, reference: T) {
        self.reference = reference;
    }
}

macro_rules! impl_drag_value {
    ($t: ident) => {
        impl DebugItem for Value<$t> {
            fn draw(&mut self, ui: &mut egui::Ui) {
                ui.horizontal(|ui| {
                    ui.label(&self.name);

                    let mut text = self.reference.to_string();
                    ui.add(egui::TextEdit::singleline(&mut text));
                    if text == "" {
                        self.reference = 0 as $t;
                    }
                    if let Ok(value) = text.parse::<$t>() {
                        self.reference = value;
                    }
                });
            }
        }
    };
}

impl_drag_value!(f32);
impl_drag_value!(f64);
impl_drag_value!(i8);
impl_drag_value!(i16);
impl_drag_value!(i32);
impl_drag_value!(i64);
impl_drag_value!(u8);
impl_drag_value!(u16);
impl_drag_value!(u32);
impl_drag_value!(u64);

impl DebugItem for Value<bool> {
    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.reference, &self.name);
    }
}
