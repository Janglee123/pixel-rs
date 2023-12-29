
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct VertexInput {
    // @builtin(position) clip_position: vec4<f32>, 
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @builtin(instance_index) instance_index: u32,
}

struct TransformUniform {
    transform: mat3x3<f32>
}

struct SpriteData {
    color: vec3<f32>,
    transform: mat3x3<f32>
}

@group(0) @binding(0) var<storage, read> sprite_data: array<SpriteData>;
@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;
@group(2) @binding(0) var<uniform> projection: mat3x3<f32>;

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;

    out.uv = vec2<f32>(in.position.x, in.position.y) * 0.5 + 0.5; 

    var proj = projection * sprite_data[in.instance_index].transform;

    out.clip_position = vec4<f32>( proj * in.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.uv);
}