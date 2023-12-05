
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
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
@group(0) @binding(1) var<uniform> tile_size: vec2<f32>;
@group(0) @binding(2) var<storage, read> tile_datas: array<TileData>;
@group(0) @binding(3) var texture: texture_2d<f32>;
@group(0) @binding(4) var texture_sampler: sampler;

@group(1) @binding(0) var<uniform> projection: mat3x3<f32>;

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;
    var tile_data: TileData = tile_datas[vertex.instance_index];

    out.color = tile_data.color * vertex.color; // Color is not channing so position is not moving for sure
    var tile_pos = vertex.position * vec3<f32>(tile_size, 1.0) + vec3<f32>(tile_data.position, 0.0);
    out.clip_position = vec4<f32>(projection * transform.transform * tile_pos, 1.0);

    out.uv = vec2<f32>(vertex.position.x, vertex.position.y) + 0.5;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.uv);

    // return vec4<f32>(in.color, 1.0);
}