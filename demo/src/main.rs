use std::f32::consts::*;

use bytemuck::{Pod, Zeroable};
use petra::{
    manager::{RenderManager, SurfaceError},
    render_pipeline::{FrontFace, PrimitiveTopology},
    texture::FRAMEBUFFER,
    wgpu::{Color, PolygonMode, ShaderStages},
    Vertex,
};
use petra_math::{Quat, Vec2, Vec3};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

const FRAC_2_PI_3: f32 = FRAC_PI_3 * 2.0;

#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
#[repr(C)]
struct ColorPosVertex {
    pos: Vec2,
    color: [f32; 3],
}

#[derive(Clone, Copy, Pod, Zeroable, Default)]
#[repr(C, align(8))]
struct TriangleUniform {
    rotation: Quat,
    offset: Vec2,
    scale: f32,
    __padding: f32,
}

fn build_triangle_vertecies(
    center: Vec2,
    side_length: f32,
    initial_rotation: f32,
) -> [ColorPosVertex; 3] {
    let a_theta = initial_rotation;
    let b_theta = initial_rotation + FRAC_2_PI_3;
    let c_theta = initial_rotation - FRAC_2_PI_3;
    let a = center + side_length * Vec2::new(a_theta.cos(), a_theta.sin());
    let b = center + side_length * Vec2::new(b_theta.cos(), b_theta.sin());
    let c = center + side_length * Vec2::new(c_theta.cos(), c_theta.sin());

    [
        ColorPosVertex {
            pos: a,
            color: [1.0, 0.0, 0.0],
        },
        ColorPosVertex {
            pos: b,
            color: [0.0, 1.0, 0.0],
        },
        ColorPosVertex {
            pos: c,
            color: [0.0, 0.0, 1.0],
        },
    ]
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    let mut manager = pollster::block_on(RenderManager::new(window));

    let vertex_buffer = manager
        .buffer_builder::<ColorPosVertex>(Some("Main Vertex Buffer"))
        .vertex()
        .build_init(build_triangle_vertecies(Vec2::ZERO, 1.0, 0.0).to_vec());

    let uniform_buffer = manager
        .buffer_builder::<TriangleUniform>(Some("Rotation Buffer"))
        .uniform()
        .copy_dst()
        .build(1);

    let bind_group = manager
        .bind_group_builder(Some("Triangle Bind Group"))
        .bind_uniform_buffer::<TriangleUniform>(0, ShaderStages::VERTEX, uniform_buffer)
        .build();

    let triangle_shader = manager.register_shader(include_str!("../shaders/triangle.wgsl"), None);
    let triangle_pipeline = manager
        .pipeline_builder(Some("triangle pipeline"))
        .vertex_shader(triangle_shader, "vs_main")
        .fragment_shader(triangle_shader, "fs_main")
        .topology(PrimitiveTopology::TriangleList)
        .polygon_mode(PolygonMode::Fill)
        .front_face(FrontFace::Cw)
        .add_vertex_buffer(vertex_buffer)
        .add_bind_group(bind_group)
        .build();

    let _pass = manager
        .pass_builder(Some("Main Pass"))
        .add_pipeline(triangle_pipeline)
        .add_attachment(FRAMEBUFFER, Some(Color::BLACK), true)
        .build();

    let mut spinning = true;
    let mut state = TriangleUniform::default();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } =>
            if window_id == manager.window.id() {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Return),
                                ..
                            },
                        ..
                    } => {
                        spinning = !spinning;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => match keycode {
                        VirtualKeyCode::Left => {
                            *state.offset.x_mut() -= 0.05;
                        }
                        VirtualKeyCode::Right => {
                            *state.offset.x_mut() += 0.05;
                        }
                        VirtualKeyCode::Up => {
                            *state.offset.y_mut() += 0.05;
                        }
                        VirtualKeyCode::Down => {
                            *state.offset.y_mut() -= 0.05;
                        }
                        VirtualKeyCode::Space => {
                            state.scale += 0.05;
                        }
                        VirtualKeyCode::LShift => {
                            state.scale -= 0.05;
                        }
                        VirtualKeyCode::D =>
                            if !spinning {
                                state.rotation *= Quat::from_axis_angle(Vec3::Z, FRAC_PI_8 / 8.0);
                            },
                        VirtualKeyCode::A =>
                            if !spinning {
                                state.rotation *=
                                    Quat::from_axis_angle(Vec3::Z, -(FRAC_PI_8 / 8.0));
                            },
                        _ => {}
                    },
                    WindowEvent::Resized(size) => manager.resize(*size),
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size: size,
                        ..
                    } => manager.resize(**size),
                    _ => {}
                }
            },
        Event::RedrawRequested(window_id) =>
            if window_id == manager.window.id() {
                if spinning {
                    state.rotation *= Quat::from_axis_angle(Vec3::Z, FRAC_PI_8 / 24.0);
                }
                manager.write_to_buffer(uniform_buffer, &[state]);


                match manager.render() {
                    Ok(_) => {}
                    Err(SurfaceError::OutOfMemory) | Err(SurfaceError::Lost) => manager.recreate(),
                    Err(SurfaceError::Outdated) => *control_flow = ControlFlow::Exit,
                    Err(SurfaceError::Timeout) => println!("Surface timed out"),
                }
            },
        Event::MainEventsCleared => manager.window.request_redraw(),
        _ => {}
    });
}
