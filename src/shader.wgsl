struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(0)
var static_texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

// full screen quad
@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32
) -> VertexOutput {
    var out: VertexOutput;
    let x: f32 = -1.0 + f32(i32(in_vertex_index)%2) * 4.0;
    let y: f32 = f32(i32(in_vertex_index)/2) * 4.0 - 1.0;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = 0.5 * vec2<f32>(x, -y)  + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coords;
    let tex = textureSample(static_texture, texture_sampler, uv).xyz;
    return vec4<f32>(tex, 1.0);
}
