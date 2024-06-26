mod debug;
mod graphics;

use std::error::Error;

struct App<'a> {
    // Graphics Devices
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    // Graphic Pipeline
    pipeline: graphics::Pipeline,

    // Debug window renderer
    debug_renderer: debug::DebugRenderer,
    debug_window: debug::Debug,




    // Winit stuff
    window: &'a winit::window::Window,
    size: winit::dpi::PhysicalSize<u32>,
}

impl<'a> App<'a> {

    async fn new(window:&'a winit::window::Window) -> Result<Self, Box<dyn Error>> {
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
            &window,
        );

        // Setup the debug window
        let debug_window = debug::Debug::init();

        // Setup the graphics pipeline
        let pipeline = graphics::Pipeline::init(&device, &config)?;


        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            debug_renderer,
            debug_window,
            window,
            size,
        })
    }

    pub fn window(&self) -> &'a winit::window::Window {
        self.window
    }

    pub fn debug(&mut self) -> &mut debug::Debug {
        &mut self.debug_window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, _event : &winit::event::WindowEvent) -> bool {
        self.window().request_redraw();
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
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
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });


        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window().scale_factor() as f32,
        };

        self.pipeline.render(&view, &mut encoder);

        self.debug_renderer.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.window,
            &view,
            screen_descriptor,
            |ui| {
                self.debug_window.run_ui(ui);
            }
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

}

pub async fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let window = winit::window::WindowBuilder::new().build(&event_loop)?;

    let mut app = App::new(&window).await?;

    let _ = event_loop.run(move |event, ewlt| match event {
        winit::event::Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == app.window().id() => {
            if !app.input(event) {
                match event {
                    winit::event::WindowEvent::CloseRequested | winit::event::WindowEvent::KeyboardInput {
                        event: winit::event::KeyEvent {
                            logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                            ..
                        },
                        ..
                    } => ewlt.exit(),
                    winit::event::WindowEvent::Resized(physical_size) => {
                        app.debug().info(&format!("Resized to {:?}", physical_size));
                        app.resize(*physical_size);
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        app.update();
                        match app.render() {
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
                app.debug_renderer.handle_input(&mut app.window, &event)
            }
        }
        _ => {
            // Nothing to do yet
        }
    });

    println!("Exiting application");

    Ok(())
}