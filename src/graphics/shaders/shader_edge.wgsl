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

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

// VertexOutput is a struct that contains the clip position and color of a vertex.
// The @builtin attribute specifies that the clip_position is a built-in variable.
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) position: vec3<f32>,
};

const SHAPE_MIN_SIZE: vec3<f32> = vec3<f32>(-10.0, -10.0, -10.0);
const SHAPE_MAX_SIZE: vec3<f32> = vec3<f32>(10.0, 10.0, 10.0);
const epsilon: f32 = 0.2;
// const SHAPE_MIN_SIZE: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
// const SHAPE_MAX_SIZE: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
// const epsilon: f32 = 0.05;


fn is_on_edge(positions: vec3<f32>) -> bool {
    if positions.x == SHAPE_MIN_SIZE.x || positions.x == SHAPE_MAX_SIZE.x {
        return true;
    }
    // return false;
    return true;
}

fn compare(position: vec3<f32>, side: i32) -> bool {
    return abs(position[side] - SHAPE_MIN_SIZE[side]) < epsilon || abs(position[side] - SHAPE_MAX_SIZE[side]) < epsilon;
}

fn check_on_side(position: vec3<f32>, side: i32) -> bool {
    // return abs(position[side] - SHAPE_MIN_SIZE[side]) < epsilon || abs(position[side] - SHAPE_MAX_SIZE[side]) < epsilon;
    return compare(position, side)
    && (
        (compare(position, (side + 1) % 3) && !compare(position, (side + 2) % 3))
    ||  (!compare(position, (side + 1) % 3) && compare(position, (side + 2) % 3))
    ||  (compare(position, (side + 1) % 3) && compare(position, (side + 2) % 3))
    );
}

// Vertex shader
// This shader is used to transform the vertices of the triangle from model space to clip space.
@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;

    // If the model is on the edge of the shape, the color is set to black.
    // if (is_on_edge(model.position)) {
    //     out.color = vec3<f32>(0.0, 0.0, 0.0);
    // } else {
    //     out.color = model.color;
    // }
    out.color = model.color;
    out.position = model.position;

    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader
// This shader is used to compute the color of each pixel of the triangle.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (
        check_on_side(in.position, 0)
    ||  check_on_side(in.position, 1)
    ||  check_on_side(in.position, 2)
    ) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return vec4<f32>(in.color, 1.0);
}
