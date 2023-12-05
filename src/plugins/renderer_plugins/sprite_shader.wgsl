
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
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
@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;
@group(2) @binding(0) var<uniform> projection: mat3x3<f32>;

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;

    out.uv = vec2<f32>(in.position.x, in.position.y) * 0.5 + 0.5; 

    var proj = projection * transform.transform;

    out.clip_position = vec4<f32>( proj * in.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.uv);
}