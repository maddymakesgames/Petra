struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) text_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) text_coords: vec2<f32>,
};


struct ObjectUniform {
    rotation: vec4<f32>,
    offset: vec2<f32>,
    scale: f32,
};

@group(0)
@binding(0)
var<uniform> state: ObjectUniform;

@group(0)
@binding(1)
var r_color: texture_2d<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let v = vec3(input.pos, 1.0);
    let q = state.rotation;
    let tmp = cross(q.xyz, v) + q.w * v;
    let pos = v + 2.0 * cross(q.xyz, tmp);
    out.pos = vec4((pos + vec3(state.offset, 0.0)) * state.scale, 1.0);
    out.color = input.color;
    out.text_coords = input.text_coords;
    return out;
}
fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let k = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + k.xyz) * 6.0 - k.www);
    let a = clamp(vec3(p) - k.xxx, vec3(0.0), vec3(1.0));
    return c.z * mix(k.xxx, a, c.y);
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_coords = vec2<i32>(in.text_coords * vec2<f32>(textureDimensions(r_color)));
    let color = textureLoad(r_color, tex_coords, 0).r;
    return vec4(hsv2rgb(vec3(color, 1.0, 1.0)), 1.0);
}