use bytemuck::{Pod, Zeroable};
use render_lib::{
    manager::{RenderManager, SurfaceError},
    render_pipeline::{FrontFace, PrimitiveTopology},
    texture::FRAMEBUFFER,
    Color,
    Vertex,
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
#[repr(C)]
struct ColorPosVertex {
    pos: [f32; 2],
    color: [f32; 3],
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    let mut manager = pollster::block_on(RenderManager::new(window));

    let vertex_buffer = manager
        .buffer_builder::<ColorPosVertex>(Some("Main Vertex Buffer"))
        .vertex()
        .build_init(vec![
            ColorPosVertex {
                pos: [0.0, 1.0],
                color: [1.0, 0.0, 0.0],
            },
            ColorPosVertex {
                pos: [-1.0, -1.0],
                color: [0.0, 1.0, 0.0],
            },
            ColorPosVertex {
                pos: [1.0, -1.0],
                color: [0.0, 0.0, 1.0],
            },
        ]);

    let triangle_shader = manager.register_shader(include_str!("../shaders/triangle.wgsl"), None);
    let triangle_pipeline = manager
        .pipeline_builder(Some("triangle pipeline"))
        .vertex_shader(triangle_shader, "vs_main")
        .fragment_shader(triangle_shader, "fs_main")
        .topology(PrimitiveTopology::TriangleList)
        .front_face(FrontFace::Cw)
        .add_vertex_buffer(vertex_buffer)
        .build();

    let _pass = manager
        .pass_builder(Some("Main Pass"))
        .add_pipeline(triangle_pipeline)
        .add_attachment(FRAMEBUFFER, Some(Color::BLACK), true)
        .build();

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
