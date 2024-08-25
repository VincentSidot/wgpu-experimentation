use std::default;

use cgmath::{Deg, InnerSpace, SquareMatrix};
use wgpu::util::DeviceExt;

pub(crate) const DEFAULT_CAMERA_POSITION: [f32; 3] = [-11.0, 15.0, 20.0];
pub(crate) const DEFAULT_CAMERA_YAW: cgmath::Deg<f32> = cgmath::Deg(-60.0);
pub(crate) const DEFAULT_CAMERA_PITCH: cgmath::Deg<f32> = cgmath::Deg(-35.0);
pub(crate) const DEFAULT_CAMERA_SPEED: f32 = 7.0;
pub(crate) const DEFAULT_CAMERA_SENSITIVITY: f32 = 2.0;
pub(crate) const DEFAULT_CAMERA_ZOOM_SENSITIVITY: f32 = 2.5;

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_2_PI - 0.001;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Debug)]
pub struct Camera {
    position: cgmath::Point3<f32>,
    yaw: cgmath::Rad<f32>,
    pitch: cgmath::Rad<f32>,
}

pub struct Projection {
    aspect: f32,
    fovy: cgmath::Rad<f32>,
    znear: f32,
    zfar: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

pub struct CameraBuffer {
    pub buffer: wgpu::Buffer,
    pub uniform: CameraUniform,
    pub camera: Camera,
    pub projection: Projection,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    pub has_been_updated: bool,
}

pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
    zoom_senstivity: f32,
}

impl Camera {
    pub fn new<
        P: Into<cgmath::Point3<f32>>,
        Y: Into<cgmath::Rad<f32>>,
        R: Into<cgmath::Rad<f32>>,
    >(
        position: P,
        yaw: Y,
        pitch: R,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn default() -> Self {
        Self {
            position: DEFAULT_CAMERA_POSITION.into(),
            yaw: DEFAULT_CAMERA_YAW.into(),
            pitch: DEFAULT_CAMERA_PITCH.into(),
        }
    }

    fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();

        cgmath::Matrix4::look_to_rh(
            self.position,
            cgmath::Vector3::new(
                cos_yaw * cos_pitch,
                sin_pitch,
                sin_yaw * cos_pitch,
            ),
            cgmath::Vector3::unit_y(),
        )
    }

    fn set_position<P: Into<cgmath::Point3<f32>>>(&mut self, position: P) {
        self.position = position.into();
    }

    fn set_yaw<Y: Into<cgmath::Rad<f32>>>(&mut self, yaw: Y) {
        self.yaw = yaw.into();
    }

    fn set_pitch<P: Into<cgmath::Rad<f32>>>(&mut self, pitch: P) {
        self.pitch = pitch.into();
    }
}

impl Projection {
    pub fn new<F: Into<cgmath::Rad<f32>>>(
        width: u32,
        height: u32,
        fovy: F,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX
            * cgmath::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(
        &mut self,
        camera: &Camera,
        projection: &Projection,
    ) {
        self.view_proj =
            (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

impl CameraController {
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity;
    }

    pub fn set_zoom_sensitivity(&mut self, zoom_sensitivity: f32) {
        self.zoom_senstivity = zoom_sensitivity;
    }

    pub fn process_keyboard(
        &mut self,
        key: winit::keyboard::KeyCode,
        state: winit::event::ElementState,
    ) -> bool {
        let amount = if state == winit::event::ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            winit::keyboard::KeyCode::KeyW
            | winit::keyboard::KeyCode::ArrowUp => {
                self.amount_forward = amount;
                true
            }
            winit::keyboard::KeyCode::KeyS
            | winit::keyboard::KeyCode::ArrowDown => {
                self.amount_backward = amount;
                true
            }
            winit::keyboard::KeyCode::KeyA
            | winit::keyboard::KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            winit::keyboard::KeyCode::KeyD
            | winit::keyboard::KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            winit::keyboard::KeyCode::Space => {
                self.amount_up = amount;
                true
            }
            winit::keyboard::KeyCode::ShiftLeft => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &winit::event::MouseScrollDelta) {
        self.scroll += match delta {
            winit::event::MouseScrollDelta::LineDelta(_, scroll) => {
                scroll * 100.0
            }
            winit::event::MouseScrollDelta::PixelDelta(
                winit::dpi::PhysicalPosition { y: scroll, .. },
            ) => *scroll as f32,
        };
    }

    pub fn update_camera(
        &mut self,
        camera_buffer: &mut CameraBuffer,
        dt: std::time::Duration,
    ) {
        let camera = &mut camera_buffer.camera;

        if self.amount_forward == 0.0
            && self.amount_backward == 0.0
            && self.amount_left == 0.0
            && self.amount_right == 0.0
            && self.amount_up == 0.0
            && self.amount_down == 0.0
            && self.rotate_horizontal == 0.0
            && self.rotate_vertical == 0.0
            && self.scroll == 0.0
        {
            return;
        } else {
            camera_buffer.has_been_updated = true;
        }

        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = cgmath::Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = cgmath::Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward
            * (self.amount_forward - self.amount_backward)
            * self.speed
            * dt;
        camera.position +=
            right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward = cgmath::Vector3::new(
            pitch_cos * yaw_cos,
            pitch_sin,
            pitch_cos * yaw_sin,
        )
        .normalize();

        // If build target is macos, the scroll is inverted.
        #[cfg(target_os = "macos")]
        let scrollward = -scrollward;

        camera.position += scrollward * self.scroll * self.zoom_senstivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y +=
            (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw +=
            cgmath::Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch +=
            cgmath::Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non-cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -cgmath::Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -cgmath::Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > cgmath::Rad(SAFE_FRAC_PI_2) {
            camera.pitch = cgmath::Rad(SAFE_FRAC_PI_2);
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed: DEFAULT_CAMERA_SPEED,
            sensitivity: DEFAULT_CAMERA_SENSITIVITY,
            zoom_senstivity: DEFAULT_CAMERA_ZOOM_SENSITIVITY,
        }
    }
}

impl CameraBuffer {
    pub fn reset_camera(&mut self) {
        self.camera = Camera::default();
    }

    pub fn get_camera_info(&self) -> (f32, f32, f32, f32, f32) {
        let yaw: Deg<f32> = self.camera.yaw.into();
        let pitch: Deg<f32> = self.camera.pitch.into();
        (
            self.camera.position[0],
            self.camera.position[1],
            self.camera.position[2],
            yaw.0,
            pitch.0,
        )
    }

    pub fn init(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let mut uniform = CameraUniform::new();
        let camera = Camera::default();
        let projection = Projection::new(
            config.width,
            config.height,
            cgmath::Deg(60.0),
            0.1,
            100.0,
        );
        uniform.update_view_proj(&camera, &projection);
        let buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        CameraBuffer {
            buffer,
            uniform,
            camera,
            projection,
            bind_group,
            bind_group_layout,
            has_been_updated: false,
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        if !self.has_been_updated {
            return;
        }
        self.uniform
            .update_view_proj(&self.camera, &self.projection);
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
        self.has_been_updated = false;
    }

    pub fn set_position<P: Into<cgmath::Point3<f32>>>(&mut self, position: P) {
        self.camera.set_position(position);
        self.has_been_updated = true;
    }

    pub fn set_yaw<Y: Into<cgmath::Rad<f32>>>(&mut self, yaw: Y) {
        self.camera.set_yaw(yaw);
        self.has_been_updated = true;
    }

    pub fn set_pitch<P: Into<cgmath::Rad<f32>>>(&mut self, pitch: P) {
        self.camera.set_pitch(pitch);
        self.has_been_updated = true;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.projection.resize(width, height);
        self.has_been_updated = true;
    }
}
