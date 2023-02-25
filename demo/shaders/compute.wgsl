// Mandelbrot shader adapted from: https://github.com/Arukiap/Mandelbrot

@group(0)
@binding(0)
var dest: texture_storage_2d<r32float, write>;

struct State {
    offset: vec2<f32>,
    zoom: f32,
};

@group(0)
@binding(1)
var<uniform> state: State;

fn square_imaginary(num: vec2<f32>) -> vec2<f32> {
    return vec2(
        num.x * num.x - num.y * num.y,
        2.0 * num.x * num.y
    );
}

const max_iterations: i32 = 10000;

fn iterate_mandlebrot(coord: vec2<f32>) -> f32 {
    var z = vec2(0.0);
    for (var i = 0; i < max_iterations; i++) {
        z = square_imaginary(z) + coord;
        if length(z) > 2.0 {
            return log(f32(i));
        }
    }
    return 1.0;
}

@compute
@workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let norm_coords = (vec2<f32>(invocation_id.xy) + vec2(0.5)) / vec2<f32>(textureDimensions(dest));
    let c = (norm_coords - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

    let val = iterate_mandlebrot(c * state.zoom + state.offset);
    textureStore(dest, vec2<i32>(invocation_id.xy), vec4(1.0 - val, 0.0, 0.0, 1.0));
}