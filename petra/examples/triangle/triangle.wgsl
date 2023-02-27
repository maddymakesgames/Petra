// The data that is sent in with our vertex
// The locations will be the order the fields are defined in the rust struct
struct VertexInput {
    @location(0)
    pos: vec2<f32>,
    @location(1)
    color: vec3<f32>,
}

// The output data from our vertex shader
// We need to have a vec4<f32> for @builtin(position)
// Any other fields will be interpolated and be accessible to our fragment shader
struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
// Our vertex shader
// Transforms the vertex data in order to render it
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.pos = vec4(input.pos, 1.0, 1.0);
    out.color = input.color;

    return out;
}

// Our fragment shader
// Is called for every fragment inside a triangle
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.color, 1.0);
}