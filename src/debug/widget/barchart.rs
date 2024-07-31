use egui::{
    epaint::RectShape, pos2, remap, remap_clamp, Rect, RichText, Sense, Shape,
    Vec2,
};

use super::debug::DebugItem;
use crate::utils::CyclicArray;
use std::{cell::RefCell, rc::Rc};

pub struct BarChart<const N: usize, const K: usize> {
    data: CyclicArray<N, [f32; K]>,
    label: [String; K],
    max_value: f32,
    min_value: f32,
}

const BARGRAPH_HEIGHT: f32 = 40.0;
const BARGRAPH_WIDTH: f32 = 200.0;
const BAR_COLORS: [egui::Color32; 8] = [
    egui::Color32::YELLOW,
    egui::Color32::RED,
    egui::Color32::GREEN,
    egui::Color32::BLUE,
    egui::Color32::WHITE,
    egui::Color32::BROWN,
    egui::Color32::KHAKI,
    egui::Color32::GRAY,
];

impl<const N: usize, const K: usize> BarChart<N, K> {
    pub fn new(data: [[f32; K]; N], labels: [String; K]) -> Rc<RefCell<Self>> {
        let mut max_value = f32::MIN;
        let mut min_value = f32::MAX;

        for value in data.iter() {
            let value = value.iter().fold(0.0, |acc, &x| acc + x);
            if value > max_value {
                max_value = value;
            }
            if value < min_value {
                min_value = value;
            }
        }

        Rc::new(RefCell::new(Self {
            data: CyclicArray::new(data),
            label: labels,
            max_value,
            min_value,
        }))
    }

    pub fn push(&mut self, values: [f32; K]) {
        self.data.push(values);
        let value = values.iter().fold(0.0, |acc, &x| acc + x);

        if value > self.max_value {
            self.max_value = value;
        }
        if value < self.min_value {
            self.min_value = value;
        }
    }
}

impl<const N: usize, const K: usize> DebugItem for BarChart<N, K> {
    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for (i, label) in self.label.iter().enumerate() {
                        ui.add(egui::Label::new(
                            RichText::new(label.clone()).background_color(
                                BAR_COLORS[i % BAR_COLORS.len()],
                            ),
                        ));
                    }
                });
                ui.separator();
                let (rect, _) = ui.allocate_at_least(
                    Vec2::new(BARGRAPH_WIDTH, BARGRAPH_HEIGHT),
                    Sense::hover(),
                );
                let style = ui.style().noninteractive();
                let mut shapes = vec![Shape::Rect(RectShape::new(
                    rect,
                    style.rounding,
                    ui.visuals().extreme_bg_color,
                    ui.style().noninteractive().bg_stroke,
                ))];

                let rect = rect.shrink(4.0);
                let half_bar_width = rect.width() / N as f32 * 0.5;

                for (i, values) in self.data.iter().enumerate() {
                    let x =
                        remap(i as f32, 0.0..=(N as f32 - 1.0), rect.x_range());
                    let x_min = ui.painter().round_to_pixel(x - half_bar_width);
                    let x_max = ui.painter().round_to_pixel(x + half_bar_width);
                    let mut last_height = rect.bottom();
                    for (j, value) in values.iter().enumerate() {
                        let y = remap_clamp(
                            *value / K as f32,
                            self.min_value..=self.max_value,
                            rect.bottom_up_range(),
                        );
                        let bar = Rect {
                            min: pos2(x_min, y),
                            max: pos2(x_max, last_height),
                        };

                        let fill_color = BAR_COLORS[j % BAR_COLORS.len()];

                        log::trace!("Bar: {:?}", bar);

                        shapes.push(Shape::Rect(RectShape::new(
                            bar,
                            0.0,
                            fill_color,
                            ui.style().noninteractive().fg_stroke,
                        )));

                        last_height = y;
                    }
                }

                self.data.rotate(1);
            });
        });
    }
}
