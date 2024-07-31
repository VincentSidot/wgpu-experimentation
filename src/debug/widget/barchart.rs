use egui::{
    epaint::RectShape, pos2, remap, remap_clamp, Rect, RichText, Sense, Shape,
    Vec2,
};

use super::debug::DebugItem;
use crate::utils::CyclicArray;
use std::{cell::RefCell, rc::Rc};

pub struct BarChart<const N: usize, const K: usize> {
    data: CyclicArray<N, [f32; K]>,
    labels: [String; K],
    colors: [egui::Color32; K],
    max_value: f32,
    min_value: f32,
}

const BARGRAPH_HEIGHT: f32 = 60.0;
const BARGRAPH_WIDTH: f32 = 200.0;

impl<const N: usize, const K: usize> BarChart<N, K> {
    pub fn new(
        data: [[f32; K]; N],
        labels: [String; K],
        colors: [egui::Color32; K],
    ) -> Rc<RefCell<Self>> {
        let mut max_value = f32::MIN;

        for value in data.iter() {
            let value = value.iter().fold(0.0, |acc, &x| acc + x);
            if value > max_value {
                max_value = value;
            }
        }

        Rc::new(RefCell::new(Self {
            data: CyclicArray::new(data),
            labels,
            colors,
            max_value,
            min_value: 0.0,
        }))
    }

    pub fn push(&mut self, values: [f32; K]) {
        let previous_values = self.data.push(values);
        let value = values.iter().fold(0.0, |acc, &x| acc + x);

        let previous_value =
            previous_values.iter().fold(0.0, |acc, &x| acc + x);

        if previous_value == self.max_value {
            self.max_value = f32::MIN;
            for value in self.data.iter() {
                let value = value.iter().fold(0.0, |acc, &x| acc + x);
                if value > self.max_value {
                    self.max_value = value;
                }
            }
        } else if value > self.max_value {
            self.max_value = value;
        }
    }
}

impl<const N: usize, const K: usize> DebugItem for BarChart<N, K> {
    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for (i, label) in self.labels.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new(
                                RichText::new("   ")
                                    .background_color(self.colors[i]),
                            ));
                            ui.label(label.clone());
                        });
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
                let bar_width = rect.width() / N as f32;

                for (i, values) in self.data.iter().enumerate() {
                    let x = remap(i as f32, 0.0..=N as f32, rect.x_range());
                    let x_min = ui.painter().round_to_pixel(x);
                    let x_max = ui.painter().round_to_pixel(x + bar_width);
                    let mut last_height = rect.bottom();
                    for (j, value) in values.iter().enumerate() {
                        let y = remap_clamp(
                            *value,
                            self.min_value..=self.max_value,
                            0.0..=(rect.min.y - rect.max.y).abs(),
                        );

                        let bar = Rect {
                            min: pos2(x_min, last_height - y),
                            max: pos2(x_max, last_height),
                        };

                        let fill_color = self.colors[j % self.colors.len()];

                        shapes.push(Shape::Rect(RectShape::new(
                            bar,
                            0.0,
                            fill_color,
                            egui::Stroke::NONE,
                        )));

                        last_height -= y;
                    }
                }

                self.data.rotate(1);
                ui.painter().extend(shapes);
            });
        });
    }
}
