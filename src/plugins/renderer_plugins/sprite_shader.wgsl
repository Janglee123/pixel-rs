
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>
};

struct VertexInput {
    // @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
}

struct TransformUniform {
    transform: mat3x3<f32>
}

@group(0) @binding(0) var<uniform> transform: TransformUniform;

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;

    out.color = transform.transform[2];
    out.clip_position = vec4<f32>(transform.transform * in.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}