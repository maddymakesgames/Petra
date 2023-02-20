struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOutput {
    var out: VertexOutput;
    // Weird vertex pos from vertex number code
    let x = f32(1 - i32(v_idx));
    let y = f32(i32(v_idx & 1u) * 2 - 1);
    out.pos = vec4(x, y, 0.0, 1.0);

    // Lerp r, g, and b based on distance
    let color_pos = out.pos * 0.5 + 0.5;
    let r = 1.0 - distance(color_pos.xy, vec2(0.5, 1.0));
    let b = 1.0 - distance(color_pos.xy, vec2(0.0, 0.0));
    let g = 1.0 - distance(color_pos.xy, vec2(1.0, 0.0));
    out.color = vec3(r, g, b);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}



