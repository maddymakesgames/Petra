use bytemuck::{Pod, Zeroable};
use petra::{
    manager::RenderManager,
    wgpu::{FrontFace, PrimitiveTopology},
    Vertex,
};
use petra_math::{Vec2, Vec3};
use wgpu::SurfaceError;
use winit::{
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// We need to derive Pod, Zeroable, and Vertex in order
// to ensure our data is safe to send to the gpu
#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
// Pod requires repr(C), the gpu assumes an align(8)
// Marking align(8) will give you an error if you don't have padding defined
#[repr(C, align(8))]
// The information we will be giving to each vertex
// This will be available in our shader
struct TriangleVertex {
    pos: Vec2,
    color: Vec3,
    // Empty padding that is needed due to alignment requirements for shaders
    __padding: f32,
}
fn main() {
    // Create a new window
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).expect("Error creating winit window");

    // Create the render manager
    let mut manager = pollster::block_on(RenderManager::new(window));

    // Register the shader
    let shader = manager.register_shader(include_str!("./triangle.wgsl"), Some("Triangle Shader"));

    // Create a new buffer, marking it as usable as a vertex buffer
    let triangle_buffer = manager
        .buffer_builder::<TriangleVertex>(Some("Triangle Vertex Buffer"))
        .vertex()
        // Initialize the buffer with the triangle vertices
        .build_init(TriangleVertex::triangle_vertices());

    // Create the pipeline that will use the shader
    let triangle_pipeline = manager
        .render_pipeline_builder(Some("Triangle Pipeline"))
        // Define the front face of a triangle
        // as being the one where the vertices are clockwise
        .front_face(FrontFace::Cw)
        // Say we're providing vertices in a triangle list
        .topology(PrimitiveTopology::TriangleList)
        // Add our shader and give the entrypoints
        .vertex_shader(shader, "vs_main")
        .fragment_shader(shader, "fs_main")
        // Add the vertex buffer
        .add_vertex_buffer(triangle_buffer)
        .build();

    // Create a new render pass to render the triangle
    // We don't define any color attachments and so
    // it assumes we want to render to the screen
    let _triangle_pass = manager
        .render_pass_builder(Some("Triangle Render Pass"))
        // Add our pipeline that will render the triangle
        .add_pipeline(triangle_pipeline)
        .build();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, event } =>
            if window_id == manager.window.id() {
                match event {
                    // If the window was resized we need to tell the manager
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } =>
                        manager.resize(*new_inner_size),
                    WindowEvent::Resized(size) => manager.resize(size),
                    // If the user is trying to close the program we should exit
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
        // Once we have handeled all the events we want to redraw
        Event::MainEventsCleared => manager.window.request_redraw(),
        Event::RedrawRequested(window_id) =>
            if manager.window.id() == window_id {
                // Tell the manager to render to the screen
                match manager.render() {
                    Ok(_) => {}
                    // If the surface was lost or out of memeory it is a critical error
                    Err(SurfaceError::Lost) | Err(SurfaceError::OutOfMemory) =>
                        *control_flow = ControlFlow::Exit,
                    // If the surface is outdated we can just recreate it
                    Err(SurfaceError::Outdated) => manager.recreate(),
                    // If the surface timed out we don't really care
                    Err(SurfaceError::Timeout) => println!("Surface timed out"),
                }
            },
        _ => {}
    })
}

impl TriangleVertex {
    // Provides 3 vertices that form a triangle
    fn triangle_vertices() -> Vec<TriangleVertex> {
        vec![
            TriangleVertex {
                pos: Vec2::new(0.0, 1.0),
                color: Vec3::new(1.0, 0.0, 0.0),
                __padding: 0.0,
            },
            TriangleVertex {
                pos: Vec2::new(-1.0, -1.0),
                color: Vec3::new(0.0, 1.0, 0.0),
                __padding: 0.0,
            },
            TriangleVertex {
                pos: Vec2::new(1.0, -1.0),
                color: Vec3::new(0.0, 0.0, 1.0),
                __padding: 0.0,
            },
        ]
    }
}
