use cgmath::{InnerSpace, SquareMatrix};

const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0,
    1.0,
);

#[derive(Debug)]
pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        let proj = cgmath::perspective(
            cgmath::Deg(self.fovy),
            self.aspect,
            self.znear,
            self.zfar,
        );

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn init(width: u32, height: u32) -> Self {
        Self {
            eye: cgmath::Point3::new(0.0, 1.0, 2.0),
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
            up: cgmath::Vector3::unit_y(), // y-axis is up
            aspect: width as f32 / height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

/// The camera controller is responsible for moving the camera based on user input.
enum CameraDirection {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

const fn bit_set(bits: u8, addr: u8, value: bool) -> u8 {
    if value {
        (bits & !addr) | addr
    } else {
        bits & !addr
    }
}

const fn bit_get(bits: u8, addr: u8) -> bool {
    (bits & addr) != 0
}

impl CameraDirection {
    const FORWARD: u8 = 1 << 1;
    const BACKWARD: u8 = 1 << 2;
    const LEFT: u8 = 1 << 3;
    const RIGHT: u8 = 1 << 4;
    const UP: u8 = 1 << 5;
    const DOWN: u8 = 1 << 6;

    const fn to_value(&self) -> u8 {
        match self {
            CameraDirection::Forward => CameraDirection::FORWARD,
            CameraDirection::Backward => CameraDirection::BACKWARD,
            CameraDirection::Left => CameraDirection::LEFT,
            CameraDirection::Right => CameraDirection::RIGHT,
            CameraDirection::Up => CameraDirection::UP,
            CameraDirection::Down => CameraDirection::DOWN,
        }
    }

    pub const fn contains(&self, bits: u8) -> bool {
        bit_get(bits, self.to_value())
    }

    pub fn set(&self, bits: &mut u8, value: bool) {
        *bits = bit_set(*bits, self.to_value(), value);
    }
}

pub struct CameraController {
    speed: f32,
    direction: u8,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            direction: 0,
        }
    }

    pub fn process_input(&mut self, event: &winit::event::WindowEvent) -> bool {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state,
                        physical_key:
                            winit::keyboard::PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                log::trace!("Key pressed: {:?} [{:?}]", keycode, state);
                let is_pressed = *state == winit::event::ElementState::Pressed;
                match keycode {
                    winit::keyboard::KeyCode::KeyW
                    | winit::keyboard::KeyCode::ArrowUp => {
                        self.set_direction(CameraDirection::Up, is_pressed);
                        true
                    }
                    winit::keyboard::KeyCode::KeyS
                    | winit::keyboard::KeyCode::ArrowDown => {
                        self.set_direction(CameraDirection::Down, is_pressed);
                        true
                    }
                    winit::keyboard::KeyCode::KeyA
                    | winit::keyboard::KeyCode::ArrowLeft => {
                        self.set_direction(CameraDirection::Left, is_pressed);
                        true
                    }
                    winit::keyboard::KeyCode::KeyD
                    | winit::keyboard::KeyCode::ArrowRight => {
                        self.set_direction(CameraDirection::Right, is_pressed);
                        true
                    }
                    _ => false,
                }
            }
            winit::event::WindowEvent::MouseWheel { delta, phase, .. } => {
                log::trace!("Mouse wheel: {:?} [{:?}]", delta, phase);
                return false;
                let direction = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        if *y > 0.0 {
                            CameraDirection::Forward
                        } else {
                            CameraDirection::Backward
                        }
                    }
                    winit::event::MouseScrollDelta::PixelDelta(
                        winit::dpi::PhysicalPosition { y, .. },
                    ) => {
                        if *y > 0.0 {
                            CameraDirection::Forward
                        } else {
                            CameraDirection::Backward
                        }
                    }
                };

                let value = match phase {
                    winit::event::TouchPhase::Started
                    | winit::event::TouchPhase::Moved => true,
                    winit::event::TouchPhase::Ended
                    | winit::event::TouchPhase::Cancelled => false,
                };

                self.set_direction(direction, value);

                true
            }
            _ => false,
        }
    }

    const fn contains(&self, direction: CameraDirection) -> bool {
        direction.contains(self.direction)
    }

    fn set_direction(&mut self, direction: CameraDirection, value: bool) {
        direction.set(&mut self.direction, value);
        log::debug!("Direction: {:08b}", self.direction);
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        // log::debug!("Camera direction: {:?}", self.direction);

        let forward = camera.target - camera.eye;
        let forward_normalized = forward.normalize();
        let forward_magnitude = forward.magnitude();

        if self.contains(CameraDirection::Forward)
            && forward_magnitude > self.speed
        {
            camera.eye += forward_normalized * self.speed;
        }

        if self.contains(CameraDirection::Backward) {
            camera.eye -= forward_normalized * self.speed;
        }

        if self.contains(CameraDirection::Up) {
            camera.eye += camera.up * self.speed;
        }
        if self.contains(CameraDirection::Down) {
            camera.eye -= camera.up * self.speed;
        }

        let right = forward_normalized.cross(camera.up);
        let forward = camera.target - camera.eye;
        let forward_magnitude = forward.magnitude();

        if self.contains(CameraDirection::Left) {
            camera.eye = camera.target
                - (forward + right * self.speed).normalize()
                    * forward_magnitude;
        }
        if self.contains(CameraDirection::Right) {
            camera.eye = camera.target
                - (forward - right * self.speed).normalize()
                    * forward_magnitude;
        }

        // let right = forward_normalized.cross(camera.up);

        // let forward = camera.target - camera.eye;
        // let forward_magnitude = forward.magnitude();

        // if self.is_right_pressed {
        //     camera.eye = camera.target
        //         - (forward + right * self.speed).normalize()
        //             * forward_magnitude;
        // }

        // if self.is_left_pressed {
        //     camera.eye = camera.target
        //         - (forward - right * self.speed).normalize()
        //             * forward_magnitude;
        // }
    }
}
