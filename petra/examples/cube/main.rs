use bytemuck::{Pod, Zeroable};
use petra::{
    manager::RenderManager,
    texture::{Depth, FRAMEBUFFER},
    wgpu::{
        CompareFunction,
        DepthBiasState,
        FrontFace,
        PrimitiveTopology,
        StencilState,
        SurfaceError,
    },
    Vertex,
};
use petra_math::{Mat4, Vec3};
use wgpu::{Color, ShaderStages};
use winit::{
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
#[repr(C, align(8))]
struct CubeVertex {
    pos: Vec3,
    color: Vec3,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C, align(8))]
struct ModelViewProjection {
    model: Mat4,
    proj: Mat4,
    view: Mat4,
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).expect("Error creating winit window");

    let mut manager = pollster::block_on(RenderManager::new(window));

    let shader = manager.register_shader(include_str!("./cube.wgsl"), Some("Cube Shader"));

    let (vertices, indicies) = CubeVertex::cube_verticies();

    let cube_vertex_buffer = manager
        .buffer_builder::<CubeVertex>(Some("Cube Vertex Buffer"))
        .vertex()
        .build_init(vertices);

    let cube_index_buffer = manager
        .buffer_builder::<u16>(Some("Cube Index Buffer"))
        .index()
        .build_init(indicies);

    let cube_transform_buffer = manager
        .buffer_builder::<ModelViewProjection>(Some("Cube Transform Buffer"))
        .uniform()
        .copy_dst()
        .build(1);

    let cube_transform_bind_group = manager
        .bind_group_builder(Some("Cube Transform Bind Group"))
        .bind_uniform_buffer::<ModelViewProjection>(0, ShaderStages::VERTEX, cube_transform_buffer)
        .build();

    let cube_pipeline = manager
        .render_pipeline_builder(Some("Cube Pipeline"))
        .front_face(FrontFace::Cw)
        .topology(PrimitiveTopology::TriangleList)
        .vertex_shader(shader, "vs_main")
        .fragment_shader(shader, "fs_main")
        .add_vertex_buffer(cube_vertex_buffer)
        .add_index_buffer(cube_index_buffer)
        .add_bind_group(cube_transform_bind_group)
        .depth_stencil::<Depth<f32>>(
            true,
            CompareFunction::Less,
            StencilState::default(),
            DepthBiasState::default(),
        )
        .build();

    let depth_texture = manager
        .texture_builder::<Depth<f32>>(Some("Depth texture"))
        .size_framebuffer()
        .render()
        .texture()
        .build();

    let _cube_pass = manager
        .render_pass_builder(Some("Cube Render Pass"))
        .add_color_attachment(FRAMEBUFFER, Some(Color::BLACK), true)
        .add_depth_stencil_attachment(depth_texture, Some((Some(1.0), true)), None)
        .add_pipeline(cube_pipeline)
        .build();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, event } =>
            if window_id == manager.window.id() {
                match event {
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } =>
                        manager.resize(*new_inner_size),
                    WindowEvent::Resized(size) => manager.resize(size),
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            },
        Event::MainEventsCleared => manager.window.request_redraw(),
        Event::RedrawRequested(window_id) =>
            if manager.window.id() == window_id {
                
                let theta = -std::f32::consts::FRAC_PI_4;
                let size = manager.window.inner_size();
                manager.write_to_buffer(cube_transform_buffer, &[ModelViewProjection {
                    model: Mat4::IDENTITY
                        * Mat4::roation_eular_xyz(theta, theta, theta)
                        * Mat4::scale(Vec3::fill(2.0)),
                    proj: Mat4::perspective_projection(
                        f32::to_radians(45.0),
                        size.width as f32 / size.height as f32,
                        0.1,
                        100.0,
                    ),
                    view: Mat4::look_at(Vec3::new(0.0, 0.0, 3.0), Vec3::fill(0.0), Vec3::Y),
                }]);

                match manager.render() {
                    Ok(_) => {}
                    Err(SurfaceError::Lost) | Err(SurfaceError::OutOfMemory) =>
                        *control_flow = ControlFlow::Exit,
                    Err(SurfaceError::Outdated) => manager.recreate(),
                    Err(SurfaceError::Timeout) => println!("Surface timed out"),
                }
            },
        _ => {}
    })
}


impl CubeVertex {
    #[rustfmt::skip]
    fn cube_verticies() -> (Vec<CubeVertex>, Vec<u16>) {
        (
            vec![
                CubeVertex {
                    pos: Vec3::new(-0.5, -0.5, -0.5),
                    color: Vec3::new(1.0, 1.0, 1.0),
                },
                CubeVertex {
                    pos: Vec3::new(0.5, -0.5, -0.5),
                    color: Vec3::new(0.0, 1.0, 1.0),
                },
                CubeVertex {
                    pos: Vec3::new(0.5, 0.5, -0.5),
                    color: Vec3::new(0.0, 0.0, 1.0),
                },
                CubeVertex {
                    pos: Vec3::new(-0.5, 0.5, -0.5),
                    color: Vec3::new(1.0, 0.0, 1.0),
                },
                CubeVertex {
                    pos: Vec3::new(-0.5, -0.5, 0.5),
                    color: Vec3::new(1.0, 1.0, 0.0),
                },
                CubeVertex {
                    pos: Vec3::new(0.5, -0.5, 0.5),
                    color: Vec3::new(0.0, 1.0, 0.0),
                },
                CubeVertex {
                    pos: Vec3::new(0.5, 0.5, 0.5),
                    color: Vec3::new(0.0, 0.0, 0.0),
                },
                CubeVertex {
                    pos: Vec3::new(-0.5, 0.5, 0.5),
                    color: Vec3::new(1.0, 0.0, 0.0),
                },
            ],
            vec![
                0, 1, 2, 
                2, 3, 0, 
                0, 4, 7, 
                7, 3, 0, 
                1, 5, 6, 
                6, 2, 1, 
                2, 3, 7, 
                7, 6, 2, 
                1, 0, 4, 
                4, 5, 1, 
                4, 5, 6, 
                6, 7, 4,
            ],
        )
    }
}
