mod debug;
mod graphics;

use std::{cell::RefCell, error::Error, rc::Rc, time::Duration};

use debug::ColorRef;

#[derive(Debug)]
pub enum WindowSize {
    FullScreen,
    FullScreenBorderless,
    Windowed(u32, u32),
}

impl Default for WindowSize {
    fn default() -> Self {
        WindowSize::Windowed(800, 600)
    }
}

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
    last_size: (u32, u32),

    // Winit stuff
    window: &'a winit::window::Window,
    size: winit::dpi::PhysicalSize<u32>,
}

macro_rules! elapsed_handler {
    ($item:expr, $block:expr) => {
        {
            let now = std::time::Instant::now();
            let ret = $block;
            let elapsed = now.elapsed();
            $item.borrow_mut().set(elapsed);
            ret
        }
    }
}

impl<'a> App<'a> {

    async fn new(window: &'a winit::window::Window) -> Result<Self, Box<dyn Error>> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("No suitable adapter found!")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None
            )
            .await?;
        


        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .ok_or("No sRGB format found on surface!")?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2, // TODO - what is this? (among every other thing I do not understand in this codebase)
        };
        
        surface.configure(&device, &config);

        // Setup the debug renderer
        let debug_renderer = debug::DebugRenderer::new(
            &device,
            config.format,
            None,
            1,
            window,
        );

        // Setup the debug window
        let debug_window = debug::Debug::init();

        // Setup the graphics pipeline
        let pipeline = graphics::Pipeline::init(&device, &config)?;


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
            last_size: (0, 0),
        })
    }

    pub fn window(&self) -> &winit::window::Window {
        self.window
    }

    pub fn debug(&mut self) -> &mut debug::Debug {
        &mut self.debug_window
    }

    pub fn gpu(&self) -> &GraphicalProcessUnit {
        &self.gpu
    }

    pub fn load_buffer(&mut self, vertices: &[graphics::Vertex], indices: &[u16]) {
        self.pipeline.load_buffer(&self.gpu.device, vertices, indices);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.gpu.config.width = new_size.width;
            self.gpu.config.height = new_size.height;
            self.gpu.surface.configure(&self.gpu.device, &self.gpu.config);
        }
    }

    fn input(&mut self, _event : &winit::event::WindowEvent) -> bool {
        self.window().request_redraw();
        false
    }

    fn update(&mut self) {}

    fn render<W, D>(
        &mut self,
        wgpu_time: Rc<RefCell<debug::widget::Label<Duration, W>>>,
        debug_time: Rc<RefCell<debug::widget::Label<Duration, D>>>,
    ) -> Result<(), wgpu::SurfaceError>
    where 
        W: Fn(&Duration) -> String,
        D: Fn(&Duration) -> String
    {
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

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });


        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.gpu.config.width, self.gpu.config.height],
            pixels_per_point: self.window().scale_factor() as f32,
        };

        elapsed_handler!(
            wgpu_time,
            self.pipeline.render(&view, &mut encoder)
        );

        let draw_pipeline = DrawPipeline {
            encoder: &mut encoder,
            window: self.window,
            view: &view,
            screen: &screen_descriptor,
        };


        elapsed_handler!(
            debug_time,
            self.debug_renderer.draw(
                &self.gpu,
                draw_pipeline,
                |ui| {
                    self.debug_window.run_ui(ui);
                }
            )
        );

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn set_size(&mut self, size: &WindowSize) {
        match size {
            WindowSize::FullScreen => {
                let monitor = self.window.current_monitor().ok_or("No monitor found!").unwrap();
                self.window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(
                    monitor.video_modes().next().ok_or("No video mode found!").unwrap()
                )));
                self.is_fullscreen = true;
            }
            WindowSize::FullScreenBorderless => {
                let monitor = self.window.current_monitor();
                self.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(
                    monitor
                )));
                self.is_fullscreen = true;
            }
            WindowSize::Windowed(width, height) => {
                let size = winit::dpi::PhysicalSize::new(*width, *height);
                self.window.set_fullscreen(None);
                if self.window.request_inner_size(size).is_some() {
                    self.resize(size);
                }
                self.last_size = (*width, *height);
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
            self.set_size(&WindowSize::FullScreen);
        } else {
            let (width, height) = self.last_size;
            self.set_size(&WindowSize::Windowed(width, height));
        }
    }

}

pub async fn run(size: &WindowSize) -> Result<(), Box<dyn Error>> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let window = winit::window::WindowBuilder::new().build(&event_loop)?;

    let update_time = debug::widget::Label::new(
        std::time::Duration::from_nanos(0),
        |v| format!("Update time: {:?}", v)
    );

    let wgpu_redraw = debug::widget::Label::new(
        std::time::Duration::from_nanos(0),
        |v| format!("WGPU Redraw time: {:?}", v)
    );

    let debug_redraw = debug::widget::Label::new(
        std::time::Duration::from_nanos(0),
        |v| format!("Debug Redraw time: {:?}", v)
    );

    let frame_per_second = debug::widget::Label::new(
        0,
        |v| format!("FPS: {}", v)
    );

    let color = debug::widget::ColorPicker::new(
        debug::RGBA {
            red: 0.1,
            green: 0.2,
            blue: 0.3,
            alpha: 1.0,
        },
        "Background Color" 
    );

    let color1 = debug::widget::ColorPicker::new(
        debug::widget::color::RGB {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
        },
        "Color 1" 
    );

    let color2 = debug::widget::ColorPicker::new(
        debug::RGB {
            red: 0.0,
            green: 1.0,
            blue: 0.0,
        },
        "Color 2" 
    );

    let color3 = debug::widget::ColorPicker::new(
        debug::RGB {
            red: 0.0,
            green: 0.0,
            blue: 1.0,
        },
        "Color 3" 
    );

    let mut vertices = vec![
        graphics::Vertex::new(
            [0.0, 0.5, 0.0],
            color1.borrow().get().into_rgb()
        ),
        graphics::Vertex::new(
            [-0.5, -0.5, 0.0],
            color2.borrow().get().into_rgb()
        ),
        graphics::Vertex::new(
            [0.5, -0.5, 0.0],
            color3.borrow().get().into_rgb()
        ),
    ];

    println!("{:?}", vertices);

    let indices = vec![0, 1, 2];

    let mut app = App::new(&window).await?;

    app.set_size(size);
    app.pipeline.set_background(
        {
            let color = color.borrow().get().into_rgba();
            wgpu::Color {
                r: color[0] as f64,
                g: color[1] as f64,
                b: color[2] as f64,
                a: color[3] as f64,
            }
        }
    );

    app.debug().add_debug_item(frame_per_second.clone());
    app.debug().add_debug_item(update_time.clone());
    app.debug().add_debug_item(wgpu_redraw.clone());
    app.debug().add_debug_item(debug_redraw.clone());
    app.debug().add_debug_item(color.clone());
    app.debug().add_debug_item(color1.clone());
    app.debug().add_debug_item(color2.clone());
    app.debug().add_debug_item(color3.clone());

    let mut last_instant = std::time::Instant::now();

    let _ = event_loop.run(move |event, ewlt| {
        match event {
            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app.window().id() => {
                if !app.input(event) {
                    match event {
                        winit::event::WindowEvent::KeyboardInput {
                            event: winit::event::KeyEvent {
                                logical_key: key,
                                state: winit::event::ElementState::Released, // Trigger only once
                                ..
                            },
                            ..
                        } => {
                            match key {
                                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) => ewlt.exit(),
                                winit::keyboard::Key::Named(winit::keyboard::NamedKey::F11) => {
                                    app.debug().info("Toggling fullscreen");
                                    app.set_fullscreen(!app.fullscreen());
                                }

                                _ => {}

                            }

                        }
                        winit::event::WindowEvent::CloseRequested => ewlt.exit(),
                        winit::event::WindowEvent::Resized(physical_size) => {
                            app.resize(*physical_size);
                        }
                        winit::event::WindowEvent::RedrawRequested => {
                            elapsed_handler!(
                                update_time,
                                app.update()
                            );
                            match app.render(
                                wgpu_redraw.clone(),
                                debug_redraw.clone()
                            ) {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    app.resize(app.size);
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    println!("Out of memory! (peepoSad)");
                                    ewlt.exit();
                                }
                                Err(wgpu::SurfaceError::Timeout) => {
                                    println!("Timeout! (peepoSad)");
                                }
                            }
                        }
                        _ => {
                            // Nothing to do yet
                        }
                    };
                    app.debug_renderer.handle_input(app.window, event);

                    // Fetch new color values
                    {
                        let color = color.borrow().get().into_rgba();
                        let color1 = color1.borrow().get().into_rgb();
                        let color2 = color2.borrow().get().into_rgb();
                        let color3 = color3.borrow().get().into_rgb();

                        app.pipeline.set_background(
                            wgpu::Color {
                                r: color[0] as f64,
                                g: color[1] as f64,
                                b: color[2] as f64,
                                a: color[3] as f64,
                            }
                        );

                        vertices[0].set_color(color1);
                        vertices[1].set_color(color2);
                        vertices[2].set_color(color3);

                        app.load_buffer(
                            &vertices,
                            &indices
                        )
                    }

                    let time = std::time::Instant::now();
                    let duration = time.duration_since(last_instant);
                    last_instant = time;

                    let fps = 1_000_000_000 / duration.as_nanos();
                    frame_per_second.borrow_mut().set(fps);

                }
            }
            _ => {
                // Nothing to do yet
            }
        }
    });

    println!("Exiting application");

    Ok(())
}