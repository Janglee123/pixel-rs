
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @builtin(instance_index) instance_index: u32,
}

struct BuildingData {
    color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> projection: mat3x3<f32>;
@group(1) @binding(0) var<storage, read> building_data: array<BuildingData>;

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;

    out.position = vec4<f32>(in.position, 0.0);
    out.color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}