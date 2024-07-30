use std::{cell::RefCell, rc::Rc};

use super::debug::DebugItem;

pub trait ColorRef {
    fn into_rgba(&self) -> [f32; 4];
    fn into_rgb(&self) -> [f32; 3];
}

macro_rules! color {
    (
        $s: ident,
        $reference: ident,
        $into_rgba: expr,
        $into_rgb: expr,
    ) => {
        impl ColorRef for $s {
            fn into_rgba(&self) -> [f32; 4] {
                let $reference = self;
                $into_rgba
            }

            fn into_rgb(&self) -> [f32; 3] {
                let $reference = self;
                $into_rgb
            }
        }
    };
}

#[derive(Debug, Default)]
pub struct RGB {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

#[derive(Debug, Default)]
pub struct RGBA {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Debug, Default)]
pub struct sRGB {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Default)]
pub struct sRGBA {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

color! {
    RGB,
    reference,
    [reference.red, reference.green, reference.blue, 1.0],
    [reference.red, reference.green, reference.blue],
}
color! {
    RGBA,
    reference,
    [reference.red, reference.green, reference.blue, reference.alpha],
    [reference.red, reference.green, reference.blue],
}
color! {
    sRGB,
    reference,
    [
        reference.red as f32 / 255.0,
        reference.green as f32 / 255.0,
        reference.blue as f32 / 255.0,
        1.0,
    ],
    [
        reference.red as f32 / 255.0,
        reference.green as f32 / 255.0,
        reference.blue as f32 / 255.0,
    ],
}
color! {
    sRGBA,
    reference,
    [
        reference.red as f32 / 255.0,
        reference.green as f32 / 255.0,
        reference.blue as f32 / 255.0,
        reference.alpha as f32 / 255.0,
    ],
    [
        reference.red as f32 / 255.0,
        reference.green as f32 / 255.0,
        reference.blue as f32 / 255.0,
    ],
}

pub struct ColorPicker<C> {
    reference: C,
    format: String,
}

impl<C> ColorPicker<C>
where
    C: ColorRef,
{
    pub fn new<S>(reference: C, format: S) -> Rc<RefCell<Self>>
    where
        S: ToString,
    {
        Rc::new(RefCell::new(Self {
            reference,
            format: format.to_string(),
        }))
    }

    pub fn set(&mut self, reference: C) {
        self.reference = reference;
    }

    pub fn get(&self) -> &C {
        &self.reference
    }

    pub fn as_text(&self) -> &str {
        &self.format
    }
}

macro_rules! color_debug {
    (
        $type: ident,
        $reference: ident,
        $value: ident,
        $convert: expr,
        $convert_bak: expr,
        $fun: expr,
    ) => {
        impl DebugItem for ColorPicker<$type> {
            fn draw(&mut self, ui: &mut egui::Ui) {
                let $reference = &self.reference;

                let mut $value = $convert;

                ui.horizontal(|ui| {
                    ui.label(self.as_text());
                    $fun(ui, &mut $value);
                });

                self.reference = $convert_bak;
            }
        }
    };
    (
        $type: ident,
        $reference: ident,
        $value: ident,
        $convert: expr,
        $convert_bak: expr,
        $fun: expr,
        $alpha: expr,
    ) => {
        impl DebugItem for ColorPicker<$type> {
            fn draw(&mut self, ui: &mut egui::Ui) {
                let $reference = &self.reference;

                let mut $value = $convert;

                ui.horizontal(|ui| {
                    ui.label(self.as_text());
                    $fun(ui, &mut $value, $alpha);
                });

                self.reference = $convert_bak;
            }
        }
    };
}

color_debug! {
    RGB,
    reference,
    value,
    [reference.red, reference.green, reference.blue],
    RGB {
        red: value[0],
        green: value[1],
        blue: value[2],
    },
    egui::widgets::color_picker::color_edit_button_rgb,
}

color_debug! {
    RGBA,
    reference,
    value,
    egui::ecolor::Rgba::from_rgba_premultiplied(
        reference.red,
        reference.green,
        reference.blue,
        reference.alpha
    ),
    RGBA {
        red: value.r(),
        green: value.g(),
        blue: value.b(),
        alpha: value.a(),
    },
    egui::widgets::color_picker::color_edit_button_rgba,
    egui::widgets::color_picker::Alpha::OnlyBlend,
}

color_debug! {
    sRGB,
    reference,
    value,
    [reference.red, reference.green, reference.blue],
    sRGB {
        red: value[0],
        green: value[1],
        blue: value[2],
    },
    egui::widgets::color_picker::color_edit_button_srgb,
}

color_debug! {
    sRGBA,
    reference,
    value,
    egui::ecolor::Color32::from_rgba_premultiplied(
        reference.red,
        reference.green,
        reference.blue,
        reference.alpha,
    ),
    sRGBA {
        red: value.r(),
        green: value.g(),
        blue: value.b(),
        alpha: value.a(),
    },
    egui::widgets::color_picker::color_edit_button_srgba,
    egui::widgets::color_picker::Alpha::OnlyBlend,
}
