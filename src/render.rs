use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::time::Duration;

use wgpu::core::instance;

use crate::config::{Config, WindowSizeHint};
use crate::graphics::shapes::Shape;
use crate::graphics::types::{Buffer, Instance};
use crate::{debug, elapsed_handler, graphics, scene, Scene};

pub struct GraphicalProcessUnit<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

pub struct DrawPipeline<'a> {
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub window: &'a winit::window::Window,
    pub view: &'a wgpu::TextureView,
    pub screen: &'a egui_wgpu::ScreenDescriptor,
}

pub struct Renderer<'a> {
    // Graphics Devices
    gpu: GraphicalProcessUnit<'a>,

    // Graphic Pipeline
    pub pipeline: graphics::Pipeline,
    // pub shapes: Option<&'a Vec<graphics::shapes::Shape>>,
    pub shapes: Vec<Rc<RefCell<Shape>>>,
    pub buffers: Vec<Option<Buffer>>,

    // Debug window renderer
    pub debug_renderer: Option<debug::DebugRenderer>,
    debug_window: debug::Debug,

    // Fullscreen
    is_fullscreen: bool,

    // Winit stuff
    pub window: &'a winit::window::Window,
    size: winit::dpi::PhysicalSize<u32>,

    // Config
    config: Config,

    // Mouse state
    mouse_pressed: bool,

    // Time state
    last_update_instant: std::time::Instant,
}

impl<'a> Renderer<'a> {
    pub async fn new(
        window: &'a winit::window::Window,
        app_config: Config,
        shader: &'static str,
    ) -> Result<Self, Box<dyn Error>> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: app_config.backends,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("No suitable adapter found!")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .ok_or("No sRGB format found on surface!")?;
        let selected_present_mode =
            app_config.to_wgpu_present_mode(&surface_caps)?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // present_mode: surface_caps.present_modes[0],
            present_mode: selected_present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2, // TODO - what is this? (among every other thing I do not understand in this codebase)
        };

        surface.configure(&device, &config);

        // Setup the debug renderer
        let debug_renderer = (!app_config.disable_egui).then(|| {
            debug::DebugRenderer::new(&device, config.format, None, 1, window)
        });

        // Setup the debug window
        let debug_window = debug::Debug::init();

        let gpu = GraphicalProcessUnit {
            surface,
            device,
            queue,
            config,
        };

        // Setup the graphics pipeline
        let pipeline = graphics::Pipeline::init(&gpu, shader)?;

        Ok(Self {
            gpu,
            pipeline,
            shapes: Vec::new(),
            buffers: Vec::new(),
            debug_renderer,
            debug_window,
            window,
            size,
            is_fullscreen: false,
            config: app_config,
            mouse_pressed: false,
            last_update_instant: std::time::Instant::now(),
        })
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn process_mouse_motion(&mut self, delta: (f64, f64)) {
        if self.mouse_pressed {
            self.pipeline.process_mouse_motion(delta);
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        self.window
    }

    pub fn debug(&mut self) -> &mut debug::Debug {
        &mut self.debug_window
    }

    pub fn set_shapes(&mut self, shapes: Vec<Rc<RefCell<Shape>>>) {
        self.shapes = shapes;
        self.buffers = std::iter::repeat_with(|| None)
            .take(self.shapes.len())
            .collect::<Vec<_>>();
    }

    pub fn load_shape(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        if let Some(shape) = self.shapes.get(index) {
            let shape = shape.as_ref().borrow();
            if let Some(buffer) = shape.buffer(&self.gpu.device) {
                self.buffers[index] = Some(buffer);
            }
            Ok(())
        } else {
            Err("Shape not found!".into())
        }
    }

    pub fn load_shapes(&mut self) {
        for (i, shape) in self.shapes.iter().enumerate() {
            let shape = shape.as_ref().borrow();
            if let Some(buffer) = shape.buffer(&self.gpu.device) {
                self.buffers[i] = Some(buffer);
            }
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.gpu.config.width = new_size.width;
            self.gpu.config.height = new_size.height;
            self.gpu
                .surface
                .configure(&self.gpu.device, &self.gpu.config);
            self.pipeline.resize(&self.gpu);
        }
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        // self.window().request_redraw();
        self.pipeline.process_input(event, &mut self.mouse_pressed)
    }

    pub fn update(&mut self, scenes: &mut Vec<Box<dyn Scene>>) {
        let dt = self.last_update_instant.elapsed();
        self.pipeline.update(&self.gpu.queue, dt);
        for scene in scenes.iter_mut() {
            scene.update(dt);
        }
        self.last_update_instant = std::time::Instant::now();
    }

    pub fn render(
        &mut self,
        wgpu_time: &mut Duration,
        debug_time: &mut Duration,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Render View"),
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let mut encoder = self.gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            },
        );

        elapsed_handler!(*wgpu_time => self.pipeline.render(&view, &mut encoder, self.buffers.iter().filter_map(|b| b.as_ref())));

        if self.debug_renderer.is_some() {
            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.gpu.config.width, self.gpu.config.height],
                pixels_per_point: self.window().scale_factor() as f32,
            };
            let draw_pipeline = DrawPipeline {
                encoder: &mut encoder,
                window: self.window,
                view: &view,
                screen: &screen_descriptor,
            };
            elapsed_handler!(
                *debug_time =>
                self.debug_renderer.as_mut().unwrap().draw(&self.gpu, draw_pipeline, |ui| {
                    self.debug_window.run_ui(ui);
                })
            );
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn update_size(&mut self) {
        match self.config.window_size.hint {
            WindowSizeHint::Fullscreen => {
                let monitor = self
                    .window
                    .current_monitor()
                    .ok_or("No monitor found!")
                    .unwrap();
                self.window.set_fullscreen(Some(
                    winit::window::Fullscreen::Exclusive(
                        monitor
                            .video_modes()
                            .next()
                            .ok_or("No video mode found!")
                            .unwrap(),
                    ),
                ));
                self.is_fullscreen = true;
            }
            WindowSizeHint::FullscreenBorderless => {
                let monitor = self.window.current_monitor();
                self.window.set_fullscreen(Some(
                    winit::window::Fullscreen::Borderless(monitor),
                ));
                self.is_fullscreen = true;
            }
            WindowSizeHint::Windowed => {
                let (width, height) = self.config.window_size.size;
                let size = winit::dpi::PhysicalSize::new(width, height);
                self.window.set_fullscreen(None);
                if self.window.request_inner_size(size).is_some() {
                    self.resize(size);
                }
                self.is_fullscreen = false;
            }
        }
    }

    pub fn fullscreen(&self) -> bool {
        self.is_fullscreen
    }

    pub fn set_fullscreen(&mut self, value: bool) {
        self.is_fullscreen = value;
        if value {
            self.config.window_size.hint = WindowSizeHint::Fullscreen;
        } else {
            self.config.window_size.hint = WindowSizeHint::Windowed;
        }
        self.update_size();
    }
}
