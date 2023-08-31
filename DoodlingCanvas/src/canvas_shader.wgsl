struct VertexInput {
    @location(0) vert_pos: vec2<f32>,
}
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
}



@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    in : VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.vert_pos.x, in.vert_pos.y, 0.0, 1.0);
    out.vert_pos = out.clip_position.xyz;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}