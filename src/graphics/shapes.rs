use super::{
    types::{Buffer, Instance},
    Vertex,
};
use crate::utils::shape::shape;
use cgmath::Vector3;
use wgpu::{util::DeviceExt as _, Device};

#[derive(Debug)]
pub struct Shape {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    instances: Vec<Instance>,
    should_be_reloaded: bool,
}

impl Shape {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u16>,
        instances: Vec<Instance>,
    ) -> Self {
        Self {
            vertices,
            indices,
            instances,
            should_be_reloaded: true,
        }
    }

    pub fn rect(
        min: Vector3<f32>,
        max: Vector3<f32>,
        color: [f32; 3],
        instances: Vec<Instance>,
    ) -> Self {
        let diff = max - min;
        let (vertices, indices) = shape!(
            color; // Red,
             // Cube vertices
            A => (max - Vector3::new(diff.x, 0.0, 0.0)).into(),
            B => max.into(),
            C => (max - Vector3::new(0.0,diff.y, 0.0)).into(),
            D => (max - Vector3::new(diff.x,diff.y, 0.0)).into(),
            E => (max - Vector3::new(diff.x,0.0, diff.z)).into(),
            F => (max - Vector3::new(0.0,0.0, diff.z)).into(),
            G => (max - Vector3::new(0.0,diff.y, diff.z)).into(),
            H => min.into();
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

        Self::new(vertices.to_vec(), indices.to_vec(), instances)
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u16] {
        &self.indices
    }

    pub fn instances(&self) -> &Vec<Instance> {
        return &self.instances;
    }

    pub fn set_instances(&mut self, instances: Vec<Instance>) {
        self.instances = instances;
        self.should_be_reloaded = true;
    }

    pub fn set_color(&mut self, color: [f32; 3]) -> &mut Self {
        self.vertices.iter_mut().for_each(|vertex| {
            vertex.set_color(color);
        });
        self.should_be_reloaded = true;
        self
    }

    fn load_buffer(&self, device: &wgpu::Device) -> Buffer {
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let instance_data = self
            .instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
        let instance_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let num_indices = self.indices.len() as u32;
        let num_instances = self.instances.len() as u32;
        Buffer {
            vertex_buffer,
            instance_buffer,
            index_buffer,
            num_indices,
            num_instances,
        }
    }

    pub fn buffer(&self, device: &Device) -> Option<Buffer> {
        if self.should_be_reloaded {
            Some(self.load_buffer(&device))
        } else {
            None
        }
    }
}
