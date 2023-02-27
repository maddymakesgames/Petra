use std::f32::consts::*;

use bytemuck::{Pod, Zeroable};
use petra::{
    manager::{RenderManager, SurfaceError},
    render_pipeline::{FrontFace, PrimitiveTopology},
    texture::FRAMEBUFFER,
    wgpu::{Color, ShaderStages, StorageTextureAccess, TextureSampleType, TextureViewDimension},
    Vertex,
};
use petra_math::{Quat, Vec2, Vec3};
use winit::{
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

const FRAC_2_PI_3: f32 = FRAC_PI_3 * 2.0;

#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
#[repr(C, align(8))]
struct ColorPosVertex {
    pos: Vec3,
    color: [f32; 3],
}

#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
#[repr(C, align(8))]
struct ColorPosTexVertex {
    pos: Vec3,
    color: [f32; 3],
    text_pos: Vec2,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C, align(8))]
struct TriangleUniform {
    rotation: Quat,
    offset: Vec2,
    scale: f32,
    __padding: f32,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C, align(8))]
struct ComputeUniform {
    offset: Vec2,
    zoom: f32,
    __padding: f32,
}


impl Default for TriangleUniform {
    fn default() -> Self {
        TriangleUniform {
            rotation: Quat::IDENTITY,
            offset: Vec2::ZERO,
            scale: 0.5,
            __padding: 0.0,
        }
    }
}

fn build_triangle_vertecies(
    center: Vec3,
    side_length: f32,
    initial_rotation: f32,
) -> [ColorPosVertex; 3] {
    let a_theta = initial_rotation;
    let b_theta = initial_rotation + FRAC_2_PI_3;
    let c_theta = initial_rotation - FRAC_2_PI_3;
    let a = center + side_length * Vec3::new(a_theta.cos(), a_theta.sin(), 0.5);
    let b = center + side_length * Vec3::new(b_theta.cos(), b_theta.sin(), 0.5);
    let c = center + side_length * Vec3::new(c_theta.cos(), c_theta.sin(), 0.5);

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
            color: [1.0, 0.0, 0.0],
        },
    ]
}

fn build_quad_vertecies() -> ([ColorPosTexVertex; 4], [u16; 6]) {
    (
        [
            ColorPosTexVertex {
                pos: Vec3::new(1.0, 1.0, 0.1),
                color: [1.0, 0.0, 0.0],
                text_pos: Vec2::new(1.0, 1.0),
            },
            ColorPosTexVertex {
                pos: Vec3::new(-1.0, 1.0, 0.1),
                color: [1.0, 0.0, 0.0],
                text_pos: Vec2::new(0.0, 1.0),
            },
            ColorPosTexVertex {
                pos: Vec3::new(-1.0, -1.0, 0.1),
                color: [0.0, 1.0, 1.0],
                text_pos: Vec2::new(0.0, 0.0),
            },
            ColorPosTexVertex {
                pos: Vec3::new(1.0, -1.0, 0.1),
                color: [0.0, 1.0, 0.0],
                text_pos: Vec2::new(1.0, 0.0),
            },
        ],
        [0, 1, 2, 2, 3, 0],
    )
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    let mut manager = pollster::block_on(RenderManager::new(window));

    manager
        .render_pass_builder(Some("Clear Pass"))
        .add_color_attachment(FRAMEBUFFER, Some(Color::BLACK), true)
        .build();

    let compute_shader = manager.register_shader(include_str!("../shaders/compute.wgsl"), None);
    let compute_texture = manager
        .texture_builder::<f32>(Some("Compute Storage Texture"))
        .size_2d(1024, 1024)
        .texture()
        .storage()
        .build();

    let compute_buffer = manager
        .buffer_builder::<ComputeUniform>(Some("Compute buffer"))
        .uniform()
        .copy_dst()
        .build(1);

    let compute_bind_group = manager
        .bind_group_builder(Some("Compute Bind Group"))
        .bind_storage_texture(
            0,
            ShaderStages::COMPUTE,
            StorageTextureAccess::WriteOnly,
            TextureViewDimension::D2,
            compute_texture,
        )
        .bind_uniform_buffer::<ComputeUniform>(1, ShaderStages::COMPUTE, compute_buffer)
        .build();
    let compute_pipeline = manager
        .compute_pipeline_builder(Some("Compute Pipeline"))
        .add_bind_group(compute_bind_group)
        .set_shader(compute_shader, "cs_main")
        .work_groups([1024 / 8, 1024 / 8, 1])
        .build();

    manager
        .compute_pass_builder(Some("Compute Pass"))
        .add_pipeline(compute_pipeline)
        .build();

    let triangle_vert_buffer = manager
        .buffer_builder::<ColorPosVertex>(Some("Triangle Vertex Buffer"))
        .vertex()
        .build_init(build_triangle_vertecies(Vec3::ZERO, 1.0, 0.0).to_vec());

    let triangle_state_buffer = manager
        .buffer_builder::<TriangleUniform>(Some("Rotation Buffer"))
        .uniform()
        .copy_dst()
        .build(1);

    let bind_group = manager
        .bind_group_builder(Some("Triangle Bind Group"))
        .bind_uniform_buffer::<TriangleUniform>(0, ShaderStages::VERTEX, triangle_state_buffer)
        .bind_texture(
            1,
            ShaderStages::FRAGMENT,
            TextureSampleType::Float { filterable: false },
            TextureViewDimension::D2,
            false,
            compute_texture,
        )
        .build();

    let triangle_shader = manager.register_shader(include_str!("../shaders/triangle.wgsl"), None);
    let _triangle_pipeline = manager
        .render_pipeline_builder(Some("triangle pipeline"))
        .vertex_shader(triangle_shader, "vs_main")
        .fragment_shader(triangle_shader, "fs_main")
        .topology(PrimitiveTopology::TriangleList)
        .front_face(FrontFace::Cw)
        .add_vertex_buffer(triangle_vert_buffer)
        .add_bind_group(bind_group)
        .build();

    let (quad_verts, quad_idx) = build_quad_vertecies();
    let quad_vert_buffer = manager
        .buffer_builder::<ColorPosTexVertex>(Some("Quad Vertex Buffer"))
        .vertex()
        .build_init(quad_verts.to_vec());

    let quad_idx_buffer = manager
        .buffer_builder::<u16>(Some("Quad Index Buffer"))
        .index()
        .build_init(quad_idx.to_vec());

    let quad_shader = manager.register_shader(include_str!("../shaders/quad.wgsl"), None);
    let quad_pipeline = manager
        .render_pipeline_builder(Some("Quad pipeline"))
        .vertex_shader(quad_shader, "vs_main")
        .fragment_shader(quad_shader, "fs_main")
        .topology(PrimitiveTopology::TriangleList)
        .front_face(FrontFace::Cw)
        .add_index_buffer(quad_idx_buffer)
        .add_vertex_buffer(quad_vert_buffer)
        .add_bind_group(bind_group)
        .build();

    manager
        .render_pass_builder(Some("Main Pass"))
        .add_pipeline(quad_pipeline)
        // .add_pipeline(triangle_pipeline)
        .add_color_attachment(FRAMEBUFFER, None, true)
        .build();

    let mut spinning = true;
    let mut shape_state = TriangleUniform::default();
    let mut fractal_state = ComputeUniform::zeroed();
    let mut modifiers = ModifiersState::empty();
    fractal_state.zoom = 1.0;

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
                    WindowEvent::ModifiersChanged(modifier) => modifiers = *modifier,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => match keycode {
                        VirtualKeyCode::Left =>
                            if modifiers.ctrl() {
                                *fractal_state.offset.x_mut() -= fractal_state.zoom;
                            } else {
                                *shape_state.offset.x_mut() -= 0.05;
                            },
                        VirtualKeyCode::Right =>
                            if modifiers.ctrl() {
                                *fractal_state.offset.x_mut() += fractal_state.zoom;
                            } else {
                                *shape_state.offset.x_mut() += 0.05;
                            },
                        VirtualKeyCode::Up =>
                            if modifiers.ctrl() {
                                *fractal_state.offset.y_mut() += fractal_state.zoom;
                            } else {
                                *shape_state.offset.y_mut() += 0.05;
                            },
                        VirtualKeyCode::Down =>
                            if modifiers.ctrl() {
                                *fractal_state.offset.y_mut() -= fractal_state.zoom;
                            } else {
                                *shape_state.offset.y_mut() -= 0.05;
                            },
                        VirtualKeyCode::Space => {
                            shape_state.scale += 0.05;
                        }
                        VirtualKeyCode::LShift => {
                            shape_state.scale -= 0.05;
                        }
                        VirtualKeyCode::D =>
                            if !spinning {
                                shape_state.rotation *=
                                    Quat::from_axis_angle(Vec3::Z, FRAC_PI_8 / 8.0);
                            },
                        VirtualKeyCode::A =>
                            if !spinning {
                                shape_state.rotation *=
                                    Quat::from_axis_angle(Vec3::Z, -(FRAC_PI_8 / 8.0));
                            },
                        VirtualKeyCode::U => {
                            fractal_state.zoom *= 0.8;
                        }
                        VirtualKeyCode::E => {
                            fractal_state.zoom /= 0.8;
                        }
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
                    shape_state.rotation *= Quat::from_axis_angle(Vec3::Z, FRAC_PI_8 / 24.0);
                }
                manager.write_to_buffer(triangle_state_buffer, &[shape_state]);
                manager.write_to_buffer(compute_buffer, &[fractal_state]);


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
