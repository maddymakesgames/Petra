use std::{fs::OpenOptions, io::Read, path::Path};

pub use wgpu::SurfaceError;
use wgpu::{
    Backends,
    CommandEncoderDescriptor,
    Device,
    DeviceDescriptor,
    Dx12Compiler,
    Features,
    Instance,
    InstanceDescriptor,
    Label,
    Limits,
    PowerPreference,
    Queue,
    RenderPassColorAttachment,
    RenderPassDescriptor,
    RenderPipeline,
    RequestAdapterOptions,
    ShaderModuleDescriptor,
    ShaderSource,
    Surface,
    SurfaceConfiguration,
    TextureUsages,
    TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    handle::Handle,
    render_pass::{RenderPassBuilder, RenderPassIntenal},
    render_pipeline::RenderPipelineBuilder,
    shader::Shader,
    texture::FRAMEBUFFER,
};

pub struct RenderManager {
    pub window: Window,
    pub(crate) surface: Surface,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) size: PhysicalSize<u32>,
    pub(crate) passes: Vec<RenderPassIntenal>,
    pub(crate) pipelines: Vec<RenderPipeline>,
    pub(crate) shaders: Vec<Shader>,
}

impl RenderManager {
    pub async fn new(window: Window) -> Self {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            dx12_shader_compiler: Dx12Compiler::default(),
        });

        let surface = unsafe { instance.create_surface(&window).unwrap() };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Main device"),
                    features: Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults()
                    } else {
                        Limits::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.describe().srgb)
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let window_size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size: window_size,
            passes: Vec::new(),
            pipelines: Vec::new(),
            shaders: Vec::new(),
        }
    }

    pub fn pipeline_builder<'a>(&'a mut self, label: Label<'a>) -> RenderPipelineBuilder<'a> {
        RenderPipelineBuilder::new(self, label)
    }

    pub fn pass_builder<'a>(&'a mut self, label: Label<'a>) -> RenderPassBuilder<'a> {
        RenderPassBuilder::new(self, label)
    }

    pub fn register_shader(&mut self, shader: &str, label: Label<'_>) -> Handle<Shader> {
        let module = self.device.create_shader_module(ShaderModuleDescriptor {
            label,
            source: ShaderSource::Wgsl(shader.into()),
        });

        let id = self.shaders.len();

        self.shaders.push(Shader(module));

        Handle::new(id)
    }

    pub fn register_shader_file(
        &mut self,
        shader: impl AsRef<Path>,
        label: Label<'_>,
    ) -> std::io::Result<Handle<Shader>> {
        let mut file = OpenOptions::new().read(true).open(shader)?;
        let mut buf = String::with_capacity(file.metadata().map(|m| m.len() as usize).unwrap_or(0));
        file.read_to_string(&mut buf)?;
        Ok(self.register_shader(&buf, label))
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn recreate(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&self) -> Result<(), SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        let surface_view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Main Render"),
            });

        for pass_desc in &self.passes {
            let mut pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: pass_desc.name.as_deref(),
                color_attachments: &pass_desc
                    .attachments
                    .iter()
                    .map(|(t, op)| {
                        let view = if *t == FRAMEBUFFER {
                            &surface_view
                        } else {
                            unimplemented!("Can't load arbitrary textures yet")
                        };
                        Some(RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: *op,
                        })
                    })
                    .collect::<Vec<_>>(),
                depth_stencil_attachment: None,
            });

            for pipeline in &pass_desc.pipelines {
                let pipeline = self
                    .pipelines
                    .get(pipeline.index())
                    .expect("Invalid RenderPipelineHandle in a render pass");
                pass.set_pipeline(pipeline);
                // TODO:
                // Add Vertex buffers, Uniform buffers, ect. to the pass before drawing
                pass.draw(0 .. 3, 0 .. 1);
            }
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}
