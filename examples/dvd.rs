use std::{cell::RefCell, rc::Rc};

use cgmath::Vector3;
use test_wgpu::{
    widget::{debug::DebugItem, Label},
    App, Config, Instance, Logger, Scene, Shape,
};

struct MyApp {
    shapes: Vec<Rc<RefCell<Shape>>>,
    center: Vector3<f32>,
    velocity: Rc<RefCell<Label<Vector3<f32>>>>,
    acceleration: Rc<RefCell<Label<Vector3<f32>>>>,
    min_point: Vector3<f32>,
    max_point: Vector3<f32>,
    size: Vector3<f32>,
}

impl MyApp {
    fn new() -> Self {
        let min_point = Vector3::new(-10.0, -10.0, -10.0);
        let max_point = Vector3::new(10.0, 10.0, 10.0);
        let size = Vector3::new(1.0, 1.0, 1.0);
        let center = Vector3::new(0.0, 0.0, 0.0);
        let velocity = Vector3::new(2.5, 1.0, 0.3);
        let acceleration = Vector3::new(0.2, 0.2, 0.2);

        let velocity = Label::new(velocity, |v| {
            format!("[dp] dx: {:.2}, dy: {:.2}, dz: {:.2}", v.x, v.y, v.z)
        });
        let acceleration = Label::new(acceleration, |a| {
            format!("[dv] dx: {:.2}, dy: {:.2}, dz: {:.2}", a.x, a.y, a.z)
        });

        let inner_rect =
            Shape::rect(-size / 2.0, size / 2.0, [0.0, 0.0, 1.0], vec![]);
        let outer_rect = Shape::rect(
            max_point,
            min_point,
            [1.0, 0.0, 0.0],
            vec![Instance::identity()],
        );

        Self {
            shapes: vec![
                Rc::new(RefCell::new(inner_rect)),
                Rc::new(RefCell::new(outer_rect)),
            ],
            center,
            velocity,
            acceleration,
            min_point,
            max_point,
            size,
        }
    }

    fn tick(&mut self, dt: std::time::Duration) {
        let acceleration = self.acceleration.borrow();
        let mut velocity = self.velocity.borrow_mut();
        {
            let acceleration = acceleration.get();
            let velocity = velocity.get_mut();

            *velocity += acceleration * dt.as_secs_f32();
            self.center += *velocity * dt.as_secs_f32();

            if self.center.x > self.max_point.x - self.size.x / 2.0
                || self.center.x < self.min_point.x + self.size.x / 2.0
            {
                velocity.x *= -1.0;
            }
            if self.center.y > self.max_point.y - self.size.y / 2.0
                || self.center.y < self.min_point.y + self.size.y / 2.0
            {
                velocity.y *= -1.0;
            }
            if self.center.z > self.max_point.z - self.size.z / 2.0
                || self.center.z < self.min_point.z + self.size.z / 2.0
            {
                velocity.z *= -1.0;
            }

            let mut inner_rect = self.shapes[0].borrow_mut();
            inner_rect.set_instances(vec![Instance::new(
                self.center,
                cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
            )]);
        }
    }
}

impl Scene for MyApp {
    fn debug_item(&self) -> Vec<Rc<RefCell<dyn DebugItem>>> {
        vec![self.velocity.clone(), self.acceleration.clone()]
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.tick(dt);
    }

    fn shapes(&self) -> Vec<Rc<RefCell<Shape>>> {
        self.shapes.clone()
    }
}

fn main() {
    let config = Config::init();
    // Setup logger
    Logger::setup(config.disable_egui).expect("Failed to setup logger");
    let mut app = App::new(config);
    app.add_scene(Box::new(MyApp::new()));

    pollster::block_on(
        app.run(include_str!("../src/graphics/shaders/shader_edge.wgsl")),
    )
    .expect("Failed to run the application");
}
