mod debug;
mod graphics;
mod utils;
mod config;

use config::WindowSizeHint;
pub use config::Config;
pub use debug::widget::Logger;
use config::AppConfig;
use utils::shape::shape;



use std::{error::Error, ops::RangeInclusive, time::Duration};

use debug::ColorRef;


struct GraphicalProcessUnit<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

struct DrawPipeline<'a> {
    encoder: &'a mut wgpu::CommandEncoder,
    window: &'a winit::window::Window,
    view: &'a wgpu::TextureView,
    screen: &'a egui_wgpu::ScreenDescriptor,
}

struct App<'a> {
    // Graphics Devices
    gpu: GraphicalProcessUnit<'a>,

    // Graphic Pipeline
    pipeline: graphics::Pipeline,

    // Debug window renderer
    debug_renderer: debug::DebugRenderer,
    debug_window: debug::Debug,

    // Fullscreen
    is_fullscreen: bool,

    // Winit stuff
    window: &'a winit::window::Window,
    size: winit::dpi::PhysicalSize<u32>,

    // Config
    config: AppConfig,

    // Mouse state
    mouse_pressed: bool,

    // Time state
    last_update_instant: std::time::Instant,
}

macro_rules! elapsed_handler {
    ($item:expr => $block:expr) => {{
        let now = std::time::Instant::now();
        let ret = $block;
        let elapsed = now.elapsed();
        $item = elapsed;
        ret
    }};
}

impl<'a> App<'a> {
    async fn new(
        window: &'a winit::window::Window,
        app_config: AppConfig,
        camera_speed: f32,
        camera_sensitivity: f32,
    ) -> Result<Self, Box<dyn Error>> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
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
        let selected_present_mode = app_config.present_mode.to_wgpu_present_mode(&surface_caps)?;
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
        let debug_renderer =
            debug::DebugRenderer::new(&device, config.format, None, 1, window);

        // Setup the debug window
        let debug_window = debug::Debug::init();

        // Setup the graphics pipeline
        let pipeline = graphics::Pipeline::init(
            &device,
            &config,
            camera_speed,
            camera_sensitivity
        )?;

        Ok(Self {
            gpu: GraphicalProcessUnit {
                surface,
                device,
                queue,
                config,
            },
            pipeline,
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

    pub fn load_buffer(
        &mut self,
        vertices: &[graphics::Vertex],
        indices: &[u16],
    ) {
        self.pipeline
            .load_buffer(&self.gpu.device, vertices, indices);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.gpu.config.width = new_size.width;
            self.gpu.config.height = new_size.height;
            self.gpu
                .surface
                .configure(&self.gpu.device, &self.gpu.config);
            self.pipeline.resize(
                self.gpu.config.width,
                self.gpu.config.height,
            );
        }
    }

    fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        // self.window().request_redraw();
        self.pipeline.process_input(event, &mut self.mouse_pressed)
    }

    fn update(&mut self) {
        let dt = self.last_update_instant.elapsed();
        self.pipeline.update(&self.gpu.queue, dt);
        self.last_update_instant = std::time::Instant::now();
    }

    fn render(
        &mut self,
        wgpu_time: &mut Duration,
        debug_time: &mut Duration,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
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

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.gpu.config.width, self.gpu.config.height],
            pixels_per_point: self.window().scale_factor() as f32,
        };

        elapsed_handler!(*wgpu_time => self.pipeline.render(&view, &mut encoder));

        let draw_pipeline = DrawPipeline {
            encoder: &mut encoder,
            window: self.window,
            view: &view,
            screen: &screen_descriptor,
        };

        elapsed_handler!(
            *debug_time =>
            self.debug_renderer.draw(&self.gpu, draw_pipeline, |ui| {
                self.debug_window.run_ui(ui);
            })
        );

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn update_size(&mut self) {
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

    fn fullscreen(&self) -> bool {
        self.is_fullscreen
    }

    fn set_fullscreen(&mut self, value: bool) {
        self.is_fullscreen = value;
        if value {
            self.config.window_size.hint = WindowSizeHint::Fullscreen;
        } else {
            self.config.window_size.hint = WindowSizeHint::Windowed;
        }
        self.update_size();
    }
}

pub async fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let config = config.compute();
    let event_loop = winit::event_loop::EventLoop::new()?;
    let window = winit::window::WindowBuilder::new()
        .with_title(&config.window_title)
        .build(&event_loop)?;

    let mut wgpu_update = Duration::from_nanos(0);
    let mut egui_update = Duration::from_nanos(0);
    let mut wgpu_redraw = Duration::from_nanos(0);
    let mut egui_redraw = Duration::from_nanos(0);

    let frame_time_label = debug::widget::Label::new(
        (String::new(), Duration::from_nanos(0), 0.0),
        |x| {
            format!(
                "FPS: {} | Frame Time: {:?} | Mean Time: {:.02}ms",
                x.0, x.1, x.2
            )
        },
    );
    let frame_time = debug::widget::BarChart::new(
        [[0.0; 5]; 100],
        [
            "WGPU Update".to_string(),
            "WGPU Draw".to_string(),
            "EGUI Update".to_string(),
            "EGUI Draw".to_string(),
            "Other Time".to_string(),
        ],
        [
            egui::Color32::LIGHT_BLUE,
            egui::Color32::BLUE,
            egui::Color32::LIGHT_GREEN,
            egui::Color32::GREEN,
            egui::Color32::WHITE,
        ],
    );

    let color = debug::widget::ColorPicker::new(
        debug::RGBA {
            red: 0.1,
            green: 0.2,
            blue: 0.3,
            alpha: 1.0,
        },
        "Background Color",
    );

    let shape_color = debug::widget::ColorPicker::new(
        debug::RGB {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
        },
        "Shape Color",
    );

    let camera_speed = debug::widget::Slider::new(1.0, RangeInclusive::new(0.1, 3.0), "Camera Speed");
    let camera_sensitivity = debug::widget::Slider::new(1.0, RangeInclusive::new(0.1, 3.0), "Camera Sensitivity");

    let (mut vertices, indices) = shape!(
        shape_color.borrow().get().into_rgb(); // Red
        // Cube vertices
        A => [0.0, 1.0, 1.0],
        B => [1.0, 1.0, 1.0],
        C => [1.0, 0.0, 1.0],
        D => [0.0, 0.0, 1.0],
        E => [0.0, 1.0, 0.0],
        F => [1.0, 1.0, 0.0],
        G => [1.0, 0.0, 0.0],
        H => [0.0, 0.0, 0.0];
        // Cube indices
        // Front face
        A D C,
        A C B,
        // Back face
        E F G,
        G H E,
        // Top face
        E A B,
        B F E,
        // Bottom face
        H G C,
        C D H,
        // Left face
        A E H,
        H D A,
        // Right face
        F B C,
        C G F,
    );

    log::trace!("Vertices: {:?}", vertices);



    let mut app = {
        let camera_speed = *camera_speed.borrow().get();
        let camera_sensitivity = *camera_sensitivity.borrow().get();
        App::new(&window, config, camera_speed, camera_sensitivity).await?
    };

    app.load_buffer(&vertices, &indices);

    app.update_size();
    app.pipeline.set_background({
        let color = color.borrow().get().into_rgba();
        wgpu::Color {
            r: color[0] as f64,
            g: color[1] as f64,
            b: color[2] as f64,
            a: color[3] as f64,
        }
    });

    app.debug().add_debug_item(frame_time_label.clone());
    app.debug().add_debug_item(frame_time.clone());
    app.debug().add_debug_item(color.clone());
    app.debug().add_debug_item(shape_color.clone());
    app.debug().add_debug_item(camera_speed.clone());
    app.debug().add_debug_item(camera_sensitivity.clone());

    let mut last_instant = std::time::Instant::now();

    let mut duration_mean = 0.0;
    let mut duration_count = 0;

    let _ = event_loop.run(move |event, ewlt| {
        match event {
            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app.window().id() && !app.debug_renderer.handle_input(app.window, event) && !app.input(event) => {
                match event {
                    winit::event::WindowEvent::KeyboardInput {
                        event:
                            winit::event::KeyEvent {
                                logical_key: key,
                                state: winit::event::ElementState::Released, // Trigger only once
                                ..
                            },
                        ..
                    } => match key {
                        winit::keyboard::Key::Named(
                            winit::keyboard::NamedKey::Escape,
                        ) => ewlt.exit(),
                        winit::keyboard::Key::Named(
                            winit::keyboard::NamedKey::F11,
                        ) => {
                            log::info!("Toggling fullscreen");
                            app.set_fullscreen(!app.fullscreen());
                        }

                        _ => {}
                    },
                    winit::event::WindowEvent::CloseRequested => {
                        ewlt.exit()
                    }
                    winit::event::WindowEvent::Resized(physical_size) => {
                        app.resize(*physical_size);
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        elapsed_handler!(*(&mut wgpu_update) => app.update());
                        match app.render(&mut wgpu_redraw, &mut egui_redraw)
                        {
                            Ok(_) => {}
                            Err(
                                wgpu::SurfaceError::Lost
                                | wgpu::SurfaceError::Outdated,
                            ) => {
                                app.resize(app.size);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                println!("Out of memory! (peepoSad)");
                                ewlt.exit();
                            }
                            Err(wgpu::SurfaceError::Timeout) => {
                                log::error!("Surface Timeout!");
                            }
                        }

                        let t1 = std::time::Instant::now();
                        {
                            let color_ref = color.borrow();
                            let shape_ref = shape_color.borrow();

                            if shape_ref.has_been_updated() {
                                let color_value = shape_ref.get().into_rgb();
                                drop(shape_ref); // Drop the reference to be able to borrow mutably

                                vertices.iter_mut().for_each(|vertex| {
                                    vertex.set_color(color_value);
                                });

                                app.load_buffer(&vertices, &indices);

                                let mut shape_ref_mut = shape_color.borrow_mut();
                                shape_ref_mut.reset_updated();
                            }
                            
                            if color_ref.has_been_updated() {
                                let color_value = color_ref.get().into_rgba();
                                app.pipeline.set_background(wgpu::Color {
                                    r: color_value[0] as f64,
                                    g: color_value[1] as f64,
                                    b: color_value[2] as f64,
                                    a: color_value[3] as f64,
                                });
                                {
                                    drop(color_ref);
                                    let mut color_mut_ref = color.borrow_mut();
                                    color_mut_ref.reset_updated();
                                }
                            }

                            let speed_ref = camera_speed.borrow();
                            let sensitivity_ref = camera_sensitivity.borrow();

                            if speed_ref.has_been_updated() {
                                log::trace!("Updating camera speed");
                                app.pipeline.camera_controller.set_speed(*speed_ref.get());
                                {
                                    drop(speed_ref);
                                    let mut speed_mut_ref = camera_speed.borrow_mut();
                                    speed_mut_ref.reset_updated();
                                }
                            }

                            if sensitivity_ref.has_been_updated() {
                                log::trace!("Updating camera sensitivity");
                                app.pipeline.camera_controller.set_sensitivity(*sensitivity_ref.get());
                                {
                                    drop(sensitivity_ref);
                                    let mut sensitivity_mut_ref = camera_sensitivity.borrow_mut();
                                    sensitivity_mut_ref.reset_updated();
                                }
                            }
                        }

                        let time = std::time::Instant::now();
                        let duration = time.duration_since(last_instant);
                        duration_mean = (duration_mean
                            * duration_count as f32
                            + (1000.0 * duration.as_secs_f32()))
                            / (duration_count + 1) as f32;
                        duration_count += 1;

                        *(&mut egui_update) = time.duration_since(t1);

                        let wgpu_update_f32 =
                            wgpu_update.as_secs_f32() * 1000.0; // ms
                        let wgpu_redraw_f32 =
                            wgpu_redraw.as_secs_f32() * 1000.0; // ms
                        wgpu_redraw = Duration::from_nanos(0);
                        let egui_redraw_f32 =
                            egui_redraw.as_secs_f32() * 1000.0; // ms
                        egui_redraw = Duration::from_nanos(0);
                        let egui_update_f32 =
                            egui_update.as_secs_f32() * 1000.0; // ms

                        let other_time_f32 = (duration.as_secs_f32()
                            * 1000.0)
                            - (wgpu_update_f32
                                + wgpu_redraw_f32
                                + egui_redraw_f32
                                + egui_update_f32); // ms

                        frame_time_label.borrow_mut().set((
                            if duration.as_nanos() > 0 {
                                format!(
                                    "{}",
                                    1_000_000_000 / duration.as_nanos()
                                )
                            } else {
                                "N/A".to_string()
                            },
                            duration,
                            duration_mean,
                        ));

                        frame_time.borrow_mut().push([
                            wgpu_update_f32,
                            wgpu_redraw_f32,
                            egui_update_f32,
                            egui_redraw_f32,
                            other_time_f32,
                        ]);
                        last_instant = time;
                    }
                    
                    
                    _ => {
                        // Nothing to do yet
                    }
                };
            }
            winit::event::Event::DeviceEvent {
                event: winit::event::DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => app.process_mouse_motion(delta),
            winit::event::Event::AboutToWait => {
                // Send a redraw request
                app.window().request_redraw();
            }
            _ => {
                // Nothing to do yet
            }
        }
    });

    println!("Exiting application");

    Ok(())
}
