mod debug;
mod graphics;
mod utils;
mod config;
mod app;

use app::App;
pub use config::Config;
pub use debug::widget::Logger;
use utils::shape::shape;



use std::{error::Error, ops::RangeInclusive, time::Duration};

pub(crate) const DEFAULT_CAMERA_POSITION: [f32; 3] = [0.0, 5.0, 10.0];
pub(crate) const DEFAULT_CAMERA_YAW: cgmath::Deg<f32> = cgmath::Deg(-90.0);
pub(crate) const DEFAULT_CAMERA_PITCH: cgmath::Deg<f32> = cgmath::Deg(-20.0);

use debug::ColorRef;
macro_rules! elapsed_handler {
    ($item:expr => $block:expr) => {{
        let now = std::time::Instant::now();
        let ret = $block;
        let elapsed = now.elapsed();
        $item = elapsed;
        ret
    }};
}

pub(crate) use elapsed_handler;

pub async fn run(config: Config) -> Result<(), Box<dyn Error>> {
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

    let camera_speed = debug::widget::Slider::new(2.0, RangeInclusive::new(0.1, 5.0), "Camera Speed");
    let camera_sensitivity = debug::widget::Slider::new(2.0, RangeInclusive::new(0.1, 5.0), "Camera Sensitivity");
    let camera_zoom_sensitivity = debug::widget::Slider::new(1.0, RangeInclusive::new(0.1, 5.0), "Camera Zoom Sensitivity");

    let reset_camera_button = debug::widget::Button::new("Reset Camera");

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
        let camera_zoom_sensitivity = *camera_zoom_sensitivity.borrow().get();
        App::new(&window, config, camera_speed, camera_sensitivity, camera_zoom_sensitivity).await?
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
    app.debug().add_debug_item(camera_zoom_sensitivity.clone());
    app.debug().add_debug_item(reset_camera_button.clone());

    let mut last_instant = std::time::Instant::now();

    let mut duration_mean = 0.0;
    let mut duration_count = 0;

    let _ = event_loop.run(move |event, ewlt| {
        match event {
            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app.window().id()
                // If the debug renderer is active, we want to handle the input of egui first (if it's not handled by egui, we can handle it)
                && app.debug_renderer.as_mut().map_or(true, |debug_renderer| !debug_renderer.handle_input(app.window, event))
                // Handle the input of the app itself
                && !app.input(event) => {
                
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
                                app.resize(app.size());
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                println!("Out of memory! (peepoSad)");
                                ewlt.exit();
                            }
                            Err(wgpu::SurfaceError::Timeout) => {
                                log::error!("Surface Timeout!");
                            }
                        }

                        
                    }
                    _ => {
                        // Nothing to do yet
                    }
                };

                // Update the debug fields if the debug renderer is active
                if app.debug_renderer.is_some() {
                    let t1 = std::time::Instant::now();
                    {
    
                        color.borrow_mut().callback_update(|value| {
                            let color_value = value.into_rgba();
                            app.pipeline.set_background(wgpu::Color {
                                r: color_value[0] as f64,
                                g: color_value[1] as f64,
                                b: color_value[2] as f64,
                                a: color_value[3] as f64,
                            });
                        });
    
                        shape_color.borrow_mut().callback_update(|value| {
                            let color_value = value.into_rgb();
                            vertices.iter_mut().for_each(|vertex| {
                                vertex.set_color(color_value);
                            });
    
                            app.load_buffer(&vertices, &indices);
                        });
    
                        camera_speed.borrow_mut().callback_update(|value| app.pipeline.camera_controller.set_speed(*value));
                        camera_sensitivity.borrow_mut().callback_update(|value| app.pipeline.camera_controller.set_sensitivity(*value));
                        camera_zoom_sensitivity.borrow_mut().callback_update(|value| app.pipeline.camera_controller.set_zoom_sensitivity(*value));
                        
                        reset_camera_button.borrow_mut().callback_update(|| {
                            app.pipeline.camera.camera.set_position(DEFAULT_CAMERA_POSITION);
                            app.pipeline.camera.camera.set_yaw(DEFAULT_CAMERA_YAW);
                            app.pipeline.camera.camera.set_pitch(DEFAULT_CAMERA_PITCH);
                        });
    
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
