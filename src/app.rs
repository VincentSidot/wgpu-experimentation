use std::{ops::RangeInclusive, time::Duration};

use crate::{
    debug::{self, ColorRef as _},
    elapsed_handler,
    scene::Scene,
    Config, Renderer,
};

pub struct App {
    config: Option<Config>,
    scenes: Vec<Box<dyn Scene>>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config: Some(config),
            scenes: Vec::new(),
        }
    }

    pub fn add_scene(&mut self, scene: Box<dyn Scene>) {
        self.scenes.push(scene);
    }

    pub async fn run(
        &mut self,
        shader: &'static str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.take().ok_or("Config not set")?;
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
            |(fps, frame_time, mean_time)| {
                format!(
                    "FPS: {} | Frame Time: {:?} | Mean Time: {:.02}ms",
                    fps, frame_time, mean_time
                )
            },
        );
        let frame_time = debug::widget::BarChart::new(
            // [[0.0; 5]; 100],
            [[0.0; 4]; 100],
            [
                "WGPU Update".to_string(),
                "WGPU Draw".to_string(),
                "EGUI Update".to_string(),
                "EGUI Draw".to_string(),
                // "Other Time".to_string(),
            ],
            [
                egui::Color32::RED,
                egui::Color32::BLUE,
                egui::Color32::LIGHT_GREEN,
                egui::Color32::GREEN,
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
        let camera_speed = debug::widget::Slider::new(
            10.0,
            RangeInclusive::new(0.1, 20.0),
            "Camera Speed",
        );
        let camera_sensitivity = debug::widget::Slider::new(
            2.0,
            RangeInclusive::new(0.1, 5.0),
            "Camera Sensitivity",
        );
        let camera_zoom_sensitivity = debug::widget::Slider::new(
            2.5,
            RangeInclusive::new(0.1, 5.0),
            "Camera Zoom Sensitivity",
        );

        let reset_camera_button = debug::widget::Button::new("Reset Camera");
        let camera_info_label = debug::widget::Label::new(
            (0.0, 0.0, 0.0, 0.0, 0.0),
            |(x, y, z, yaw, pitch)| {
                // let (x, y, z, yaw, pitch) = f
                format!(
                    "[Camera] x:{:.02}, y:{:.02}, z:{:.02}, yaw:{:.02}°, pitch: {:.02}°",
                    x, y, z, yaw, pitch
                )
            },
        );

        // let outter_rect = Shape::rect(
        //     Vector3::new(1.0, 1.0, 1.0),
        //     Vector3::new(0.0, 0.0, 0.0),
        //     [1.0, 0.0, 0.0], // Red
        //     vec![Instance::identity()]
        // );
        // let innert_rect = Shape::rect(
        //     Vector3::new(0.25, 0.25, 0.25),
        //     Vector3::new(0.75, 0.75, 0.75),
        //     [0.0, 0.0, 1.0], // Blue
        //     vec![
        //         Instance::identity(),
        //         Instance::identity().with_translation([2.0, 0.0, 0.0]),
        //         Instance::identity().with_translation([-2.0, 0.0, 0.0])
        //     ],
        // );

        let mut renderer = { Renderer::new(&window, config, shader).await? };

        // *app.shapes_mut() = vec![outter_rect, innert_rect];
        // app.load_shapes();

        renderer.update_size();
        renderer.pipeline.set_background({
            let color = color.borrow().get().into_rgba();
            wgpu::Color {
                r: color[0] as f64,
                g: color[1] as f64,
                b: color[2] as f64,
                a: color[3] as f64,
            }
        });

        renderer.debug().add_debug_item(frame_time_label.clone());
        renderer.debug().add_debug_item(frame_time.clone());
        renderer.debug().add_debug_item(color.clone());
        renderer.debug().add_debug_item(camera_speed.clone());
        renderer.debug().add_debug_item(camera_sensitivity.clone());
        renderer
            .debug()
            .add_debug_item(camera_zoom_sensitivity.clone());
        renderer.debug().add_debug_item(reset_camera_button.clone());
        renderer.debug().add_debug_item(camera_info_label.clone());

        log::debug!("Scenes count: {}", self.scenes.len());
        // Add the debug items from the scenes
        for scene in &self.scenes {
            renderer.debug().add_separator();
            for item in scene.debug_item() {
                renderer.debug().add_debug_item(item.clone());
            }
        }

        // Setup the shapes from the scenes
        for scene in &self.scenes {
            renderer.set_shapes(scene.shapes());
        }
        renderer.load_shapes();
        log::debug!(
            "Renderer buffer count: {}",
            renderer
                .buffers
                .iter()
                .filter(|buffer| buffer.is_some())
                .count()
        );

        let mut last_instant = std::time::Instant::now();

        let mut duration_mean = 0.0;
        let mut duration_count = 0;

        let _ = event_loop.run(move |event, ewlt| {
            match event {
                winit::event::Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == renderer.window().id()
                    // If the debug renderer is active, we want to handle the input of egui first (if it's not handled by egui, we can handle it)
                    && renderer.debug_renderer.as_mut().map_or(true, |debug_renderer| !debug_renderer.handle_input(renderer.window, event))
                    // Handle the input of the app itself
                    && !renderer.input(event) => {

                    // Handle the window events
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
                                renderer.set_fullscreen(!renderer.fullscreen());
                            }

                            _ => {}
                        },
                        winit::event::WindowEvent::CloseRequested => {
                            ewlt.exit()
                        }
                        winit::event::WindowEvent::Resized(physical_size) => {
                            renderer.resize(*physical_size);
                        }
                        winit::event::WindowEvent::RedrawRequested => {
                            elapsed_handler!(*(&mut wgpu_update) => renderer.update(&mut self.scenes));
                            // Reload the buffers if needed
                            renderer.load_shapes();
                            match renderer.render(&mut wgpu_redraw, &mut egui_redraw)
                            {
                                Ok(_) => {}
                                Err(
                                    wgpu::SurfaceError::Lost
                                    | wgpu::SurfaceError::Outdated,
                                ) => {
                                    renderer.resize(renderer.size());
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    println!("Out of memory! (peepoSad)");
                                    ewlt.exit();
                                }
                                Err(wgpu::SurfaceError::Timeout) => {
                                    log::error!("Surface Timeout!");
                                }
                            }

                            // Update the debug fields if the debug renderer is active
                            if renderer.debug_renderer.is_some() {
                                let t1 = std::time::Instant::now();
                                {

                                    color.borrow_mut().callback_update(|value| {
                                        let color_value = value.into_rgba();
                                        renderer.pipeline.set_background(wgpu::Color {
                                            r: color_value[0] as f64,
                                            g: color_value[1] as f64,
                                            b: color_value[2] as f64,
                                            a: color_value[3] as f64,
                                        });
                                    });

                                    camera_speed.borrow_mut().callback_update(|value| renderer.pipeline.camera_controller.set_speed(*value));
                                    camera_sensitivity.borrow_mut().callback_update(|value| renderer.pipeline.camera_controller.set_sensitivity(*value));
                                    camera_zoom_sensitivity.borrow_mut().callback_update(|value| renderer.pipeline.camera_controller.set_zoom_sensitivity(*value));

                                    reset_camera_button.borrow_mut().callback_update(|| {
                                        renderer.pipeline.camera.reset_camera();
                                    });

                                }

                                camera_info_label.borrow_mut().set(renderer.pipeline.camera.get_camera_info());

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

                                // let other_time_f32 = (duration.as_secs_f32()
                                //     * 1000.0)
                                //     - (wgpu_update_f32
                                //         + wgpu_redraw_f32
                                //         + egui_redraw_f32
                                //         + egui_update_f32); // ms

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
                                    // other_time_f32,
                                ]);
                                last_instant = time;
                            }


                        }
                        _ => {
                            // Nothing to do yet
                        }
                    };

                }
                winit::event::Event::DeviceEvent {
                    event: winit::event::DeviceEvent::MouseMotion{ delta, },
                    .. // We're not using device_id currently
                } => renderer.process_mouse_motion(delta),
                winit::event::Event::AboutToWait => {
                    // Send a redraw request
                    renderer.window().request_redraw();
                }
                _ => {
                    // Nothing to do yet
                }
            }
        });

        println!("Exiting application");

        Ok(())
    }
}
