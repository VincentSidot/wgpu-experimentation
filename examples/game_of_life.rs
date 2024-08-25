use core::f32;
use std::{cell::RefCell, rc::Rc, vec};

// TODO: Re-implement this by using texture instead of shapes

use cgmath::Vector3;
use test_wgpu::{widget, App, Config, Instance, Logger, Scene, Shape};

struct GameOfLife {
    grid: Vec<bool>,
    size_grid: usize,
    size_ratio: f32,
    shape_alive: Rc<RefCell<Shape>>,
    shape_dead: Rc<RefCell<Shape>>,
    shape_frame: Rc<RefCell<Shape>>, // ...
    last_update: std::time::Instant,
    time_debug:
        Rc<RefCell<widget::Label<(std::time::Duration, std::time::Duration)>>>,
}

const ALIVE: [f32; 3] = [1.0, 1.0, 1.0];
const DEAD: [f32; 3] = [0.0, 0.0, 0.0];

impl GameOfLife {
    fn new(size_grid: usize, alive_probability: f32) -> Self {
        let size_shape = BOX_SIZE;
        let grid: Vec<bool> = (0..size_grid * size_grid)
            .map(|_| rand::random::<f32>() < alive_probability)
            .collect();
        let size_ratio = size_shape / size_grid as f32;
        let p1 = Vector3::new(-size_ratio / 2.0, -size_ratio / 2.0, 0.0);
        let p2 = Vector3::new(size_ratio / 2.0, size_ratio / 2.0, 1.0);

        let shape_alive = Shape::rect(p1, p2, ALIVE, vec![]);
        let shape_dead = Shape::rect(p1, p2, DEAD, vec![]);
        let delta = 0.5;
        let shape_frame = Shape::rect(
            Vector3::new(-delta, -delta, 0.01),
            Vector3::new(size_shape + delta, size_shape + delta, 0.99),
            [1.0, 0.0, 0.0],
            vec![Instance::identity()],
        );

        let time_debug = widget::Label::new(
            (
                std::time::Duration::from_micros(0),
                std::time::Duration::from_micros(0),
            ),
            |(a, b)| {
                format!(
                    "Update: {:.2}ms, Render: {:.2}ms",
                    a.as_micros() as f32 / 1000.0,
                    b.as_micros() as f32 / 1000.0
                )
            },
        );

        Self {
            grid,
            size_grid,
            size_ratio,
            shape_alive: Rc::new(RefCell::new(shape_alive)),
            shape_dead: Rc::new(RefCell::new(shape_dead)),
            shape_frame: Rc::new(RefCell::new(shape_frame)),
            last_update: std::time::Instant::now(),
            time_debug,
        }
    }

    fn tick(&mut self) {
        let mut new_grid = Vec::with_capacity(self.size_grid * self.size_grid);

        for (index, value) in self.grid.iter().enumerate() {
            let x = index % self.size_grid;
            let y = index / self.size_grid;
            let mut alive_neighbours = 0;

            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;

                    if nx < self.size_grid && ny < self.size_grid {
                        let neighbour_index = ny * self.size_grid + nx;
                        if self.grid[neighbour_index] {
                            alive_neighbours += 1;
                        }
                    }
                }
            }

            let new_value = match (value, alive_neighbours) {
                (true, 2) | (true, 3) => true,
                (false, 3) => true,
                _ => false,
            };

            new_grid.push(new_value);
        }
        std::mem::swap(&mut self.grid, &mut new_grid);
    }
}

impl Scene for GameOfLife {
    fn update(&mut self, _dt: std::time::Duration) {
        if self.last_update.elapsed().as_secs_f32() < TIME_STEP {
            return; // No need to update the scene
        }
        self.last_update = std::time::Instant::now();

        let t1 = std::time::Instant::now();
        self.tick();
        let t2 = std::time::Instant::now();
        let mut instances_alive = vec![];
        let mut instances_dead = vec![];
        for (index, value) in self.grid.iter().enumerate() {
            let x = index % self.size_grid;
            let y = index / self.size_grid;
            let position = Vector3::new(
                x as f32 * self.size_ratio, // + self.size_ratio / 2.0,
                y as f32 * self.size_ratio, // + self.size_ratio / 2.0,
                0.0,
            );

            if *value {
                instances_alive
                    .push(Instance::identity().with_translation(position));
            } else {
                instances_dead
                    .push(Instance::identity().with_translation(position));
            }
        }
        self.shape_alive.borrow_mut().set_instances(instances_alive);
        self.shape_dead.borrow_mut().set_instances(instances_dead);
        let t3 = std::time::Instant::now();
        self.time_debug.borrow_mut().set((t2 - t1, t3 - t2));
    }

    fn shapes(&self) -> Vec<std::rc::Rc<std::cell::RefCell<Shape>>> {
        vec![
            self.shape_alive.clone(),
            self.shape_dead.clone(),
            self.shape_frame.clone(),
        ]
    }

    fn debug_item(&self) -> Vec<Rc<RefCell<dyn widget::debug::DebugItem>>> {
        vec![self.time_debug.clone()]
    }
    // ...
}

const GRID_SIZE: usize = 300;
const TIME_STEP: f32 = 0.0 / 20.0;
const BOX_SIZE: f32 = 20.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::init();
    Logger::setup(config.disable_egui).expect("Failed to setup logger");
    let mut app = App::new(config);
    app.add_scene(Box::new(GameOfLife::new(GRID_SIZE, 0.6)));
    pollster::block_on(
        app.run(include_str!("../src/graphics/shaders/shader.wgsl")),
    )
    .expect("Failed to run the application");

    Ok(())
}
