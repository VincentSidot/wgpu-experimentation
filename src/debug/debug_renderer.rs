use egui::{epaint::Shadow, Context, Visuals};

use egui_wgpu::Renderer;
use egui_winit::State;

use crate::app::{DrawPipeline, GraphicalProcessUnit};

pub struct DebugRenderer {
    pub context: Context,
    state: State,
    renderer: Renderer,
}

impl DebugRenderer {
    pub fn new(
        device: &wgpu::Device,
        output_color_fromat: wgpu::TextureFormat,
        output_depth_format: Option<wgpu::TextureFormat>,
        msaa_samples: u32,
        window: &winit::window::Window,
    ) -> DebugRenderer {
        let egui_context = Context::default();
        let id = egui_context.viewport_id();

        const BORDER_RADIUS: f32 = 2.0;

        let visuals = Visuals {
            window_rounding: egui::Rounding::same(BORDER_RADIUS),
            window_shadow: Shadow::NONE,

            ..Visuals::dark()
        };

        egui_context.set_visuals(visuals);

        let egui_state = egui_winit::State::new(
            egui_context.clone(),
            id,
            &window,
            None,
            None,
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            device,
            output_color_fromat,
            output_depth_format,
            msaa_samples,
        );

        DebugRenderer {
            context: egui_context,
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    pub fn handle_input(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    pub fn draw(
        &mut self,
        gpu: &GraphicalProcessUnit,
        pipeline: DrawPipeline,
        run_ui: impl FnOnce(&egui::Context),
    ) {
        let raw_input = self.state.take_egui_input(pipeline.window);

        let full_output = self.context.run(raw_input, |_ui| {
            run_ui(&self.context);
        });

        self.state.handle_platform_output(
            pipeline.window,
            full_output.platform_output,
        );

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(
                &gpu.device,
                &gpu.queue,
                *id,
                image_delta,
            );
        }

        self.renderer.update_buffers(
            &gpu.device,
            &gpu.queue,
            pipeline.encoder,
            &tris,
            pipeline.screen,
        );

        let mut rpass =
            pipeline
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: pipeline.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    label: Some("Debug View Render Pass"),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

        self.renderer.render(&mut rpass, &tris, pipeline.screen);
        drop(rpass);

        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}
