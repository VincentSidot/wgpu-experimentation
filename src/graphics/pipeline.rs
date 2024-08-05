use std::error::Error;

use wgpu::util::DeviceExt;

use crate::utils::lisp::prog1;

use super::camera::{self, Camera, CameraBuffer};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

/// Implement the Vertex struct
impl Vertex {
    /// Describes the layout of the Vertex struct
    ///
    /// This is used to tell the GPU how to interpret the data in the vertex buffer
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>()
                        as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }

    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self { position, color }
    }

    // pub fn with_position(&mut self, position: [f32; 3]) -> &mut Self {
    //     self.position = position;
    //     self
    // }

    // pub fn with_color(&mut self, color: [f32; 3]) -> &mut Self {
    //     self.color = color;
    //     self
    // }

    // pub fn color(&self) -> [f32; 3] {
    //     self.color
    // }

    // pub fn position(&self) -> [f32; 3] {
    //     self.position
    // }

    // pub fn position_mut(&mut self) -> &mut [f32; 3] {
    //     &mut self.position
    // }

    // pub fn color_mut(&mut self) -> &mut [f32; 3] {
    //     &mut self.color
    // }

    // pub fn set_position(&mut self, position: [f32; 3]) {
    //     self.position = position;
    // }

    pub fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
    }
}

// const VERTICES: &[Vertex] = &[
//     Vertex {
//         position: [0.0, 0.5, 0.0],
//         color: [1.0, 0.0, 0.0],
//     },
//     Vertex {
//         position: [-0.5, -0.5, 0.0],
//         color: [0.0, 1.0, 0.0],
//     },
//     Vertex {
//         position: [0.5, -0.5, 0.0],
//         color: [0.0, 0.0, 1.0],
//     },
// ];

// const INDICES: &[u16] = &[0, 1, 2];

struct Buffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

pub struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    buffer: Option<Buffer>,
    background_color: wgpu::Color,
    camera: CameraBuffer,
    camera_controller: camera::CameraController,
}

impl Pipeline {
    // pub fn background(&self) -> wgpu::Color {
    //     self.background_color
    // }

    pub fn set_background(&mut self, color: wgpu::Color) {
        self.background_color = color;
    }

    pub fn init(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, Box<dyn Error>> {
        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shaders/shader.wgsl").into(),
                ),
            });

        let camera = CameraBuffer::init(device, config);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera.bind_group_layout()],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
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
                    // cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        Ok(Self {
            render_pipeline,
            buffer: None,
            background_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            camera,
            camera_controller: camera::CameraController::new(0.2),
        })
    }

    pub fn load_buffer(
        &mut self,
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u16],
    ) {
        log::trace!("Loading buffer");
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let num_indices = indices.len() as u32;

        self.buffer = Some(Buffer {
            vertex_buffer,
            index_buffer,
            num_indices,
        });
    }

    pub fn process_input(&mut self, event: &winit::event::WindowEvent) -> bool {
        let pinput = self.camera_controller.process_input(event);
        if pinput {
            log::trace!("Camera controller: {:?}", self.camera.camera);
        }
        pinput
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        // Update camera
        self.camera_controller
            .update_camera(&mut self.camera.camera);
        self.camera.update(queue);
    }

    pub fn render(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // Draw the buffer if it exists
        if let Some(buffer) = &self.buffer {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    self.background_color,
                                ),
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

            let vertex_buffer = &buffer.vertex_buffer;
            let index_buffer = &buffer.index_buffer;
            let num_indices = buffer.num_indices;

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        } else {
            log::error!("No buffer to render");
        }
    }
}

impl CameraBuffer {
    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.uniform.update_view_proj(&self.camera);
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}
