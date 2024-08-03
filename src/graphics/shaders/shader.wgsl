// This file contains the WebGPU Shading Language (WGSL) code for the vertex and fragment shaders.

// CameraUniform is a struct that contains the view-projection matrix.
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

// VertexInput is a struct that contains the position and color of a vertex.
// The @location attribute specifies the location of the input data.
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

// VertexOutput is a struct that contains the clip position and color of a vertex.
// The @builtin attribute specifies that the clip_position is a built-in variable.
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

// Vertex shader
// This shader is used to transform the vertices of the triangle from model space to clip space.
@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader
// This shader is used to compute the color of each pixel of the triangle.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}