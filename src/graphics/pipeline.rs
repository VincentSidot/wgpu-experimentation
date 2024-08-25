use core::num;
use std::{cell::RefCell, error::Error, rc::Rc};

use wgpu::Device;

use crate::render::GraphicalProcessUnit;

use super::{
    camera::{self, CameraBuffer},
    shapes::{self, Shape},
    types::{Buffer, Instance, InstanceRaw, Vertex},
};

pub struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    background_color: wgpu::Color,
    pub camera: CameraBuffer,
    pub camera_controller: camera::CameraController,
    depth_texture: DepthTexture,
}

struct DepthTexture {
    view: wgpu::TextureView,
}

impl Pipeline {
    pub fn set_background(&mut self, color: wgpu::Color) {
        self.background_color = color;
    }

    pub fn init(
        gpu: &GraphicalProcessUnit,
        shader: &'static str,
    ) -> Result<Self, Box<dyn Error>> {
        let shader =
            &gpu.device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Shader"),
                    source: wgpu::ShaderSource::Wgsl(shader.into()),
                });

        let camera = CameraBuffer::init(&gpu.device, &gpu.config);

        let render_pipeline_layout = &gpu.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera.bind_group_layout()],
                push_constant_ranges: &[],
            },
        );

        let render_pipeline = gpu.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc(), InstanceRaw::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gpu.config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // cull_mode: Some(wgpu::Face::Front),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                // depth_stencil: None,
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DepthTexture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            },
        );

        Ok(Self {
            render_pipeline,
            background_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            camera,
            camera_controller: camera::CameraController::default(),
            depth_texture: DepthTexture::create_depth_structure(
                gpu,
                "Depth Texture",
            ),
        })
    }

    pub fn process_input(
        &mut self,
        event: &winit::event::WindowEvent,
        mouse_pressed: &mut bool,
    ) -> bool {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            winit::event::WindowEvent::MouseInput {
                button: winit::event::MouseButton::Left,
                state,
                ..
            } => {
                *mouse_pressed = *state == winit::event::ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse_motion(&mut self, position: (f64, f64)) {
        self.camera_controller.process_mouse(position.0, position.1);
    }

    pub fn resize(&mut self, gpu: &GraphicalProcessUnit) {
        self.camera.resize(gpu.config.width, gpu.config.height);
        self.depth_texture =
            DepthTexture::create_depth_structure(&gpu, "Depth Texture");
    }

    pub fn update(&mut self, queue: &wgpu::Queue, dt: std::time::Duration) {
        // Update camera
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera.update(queue);
    }

    pub fn render<'a, I>(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        buffers: I,
    ) where
        I: Iterator<Item = &'a Buffer>,
    {
        // Draw the buffer if it exists
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.background_color),
                        // load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                // depth_stencil_attachment: None,
                depth_stencil_attachment: Some(
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    },
                ),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.camera.bind_group(), &[]);
        for buffer in buffers {
            let vertex_buffer = &buffer.vertex_buffer;
            let index_buffer = &buffer.index_buffer;
            let instance_buffer = &buffer.instance_buffer;
            let num_indices = buffer.num_indices;
            let num_instances = buffer.num_instances;

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_index_buffer(
                index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(0..num_indices, 0, 0..num_instances);
        }
    }
}

impl DepthTexture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat =
        wgpu::TextureFormat::Depth32Float; // 1.

    pub fn create_depth_structure(
        gpu: &GraphicalProcessUnit,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: gpu.config.width,
            height: gpu.config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = gpu.device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        //     address_mode_u: wgpu::AddressMode::ClampToEdge,
        //     address_mode_v: wgpu::AddressMode::ClampToEdge,
        //     address_mode_w: wgpu::AddressMode::ClampToEdge,
        //     mag_filter: wgpu::FilterMode::Linear,
        //     min_filter: wgpu::FilterMode::Linear,
        //     mipmap_filter: wgpu::FilterMode::Nearest,
        //     compare: Some(wgpu::CompareFunction::LessEqual),
        //     lod_min_clamp: 0.0,
        //     lod_max_clamp: 100.0,
        //     ..Default::default()
        // });
        Self { view }
    }
}
