#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

pub struct Buffer {
    pub vertex_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub num_instances: u32,
}

/// Implement the Vertex struct
impl Vertex {
    /// Describes the layout of the Vertex struct
    ///
    /// This is used to tell the GPU how to interpret the data in the vertex buffer
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
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

impl Instance {
    pub fn new<P, R>(position: P, rotation: R) -> Self
    where
        P: Into<cgmath::Vector3<f32>>,
        R: Into<cgmath::Quaternion<f32>>,
    {
        Self {
            position: position.into(),
            rotation: rotation.into(),
        }
    }

    pub const fn identity() -> Self {
        Self {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn with_translation<P>(mut self, position: P) -> Self
    where
        P: Into<cgmath::Vector3<f32>>,
    {
        self.position = position.into();
        self
    }

    pub fn with_rotation<R>(mut self, rotation: R) -> Self
    where
        R: Into<cgmath::Quaternion<f32>>,
    {
        self.rotation = rotation.into();
        self
    }

    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>()
                as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>()
                        as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>()
                        as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>()
                        as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
