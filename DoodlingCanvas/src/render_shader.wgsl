struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
    @location(1) vert_uv: vec2<f32>,
}



@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var vertices : array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
    );
    var uvs : array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
    );
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vertices[in_vertex_index].x , vertices[in_vertex_index].y , 0.0, 1.0);
    out.vert_pos = out.clip_position.xyz;
    out.vert_uv = uvs[in_vertex_index];
    return out;
}

@group(0) @binding(0)
var canvas: texture_2d<f32>;
@group(0)@binding(1)
var canvas_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(canvas, canvas_sampler, in.vert_pos.xy);
}