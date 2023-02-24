struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};


struct ObjectUniform {
    rotation: vec4<f32>,
    offset: vec2<f32>,
    scale: f32,
};

@group(0)
@binding(0)
var<uniform> state: ObjectUniform;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let v = vec3(input.pos, 1.0);
    let q = state.rotation;
    let tmp = cross(q.xyz, v) + q.w * v;
    let pos = v + 2.0 * cross(q.xyz, tmp);
    out.pos = vec4((pos + vec3(state.offset, 0.0)) * state.scale, 1.0);
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}



