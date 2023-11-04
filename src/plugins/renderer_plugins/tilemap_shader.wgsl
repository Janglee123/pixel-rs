
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>
};

struct VertexInput {
    // @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @builtin(instance_index) instance_index: u32
};

struct TransformUniform {
    transform: mat3x3<f32>
}

struct TileData {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> transform: TransformUniform;
@group(0) @binding(1) var<storage, read> tile_datas: array<TileData>;

@group(1) @binding(0) var<uniform> projection: mat3x3<f32>;

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;
    var tile_data: TileData = tile_datas[in.instance_index];

    out.color = in.color; // Color is not channing so position is not moving for sure
    out.clip_position = vec4<f32>(projection * transform.transform * (in.position + vec3<f32>(tile_data.position, 0.0)), 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}