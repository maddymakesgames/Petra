use std::{fs::OpenOptions, io::Read, path::Path, sync::Arc};

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
    buffer::{Buffer, BufferBuilder, BufferContents},
    handle::{Handle, Registry},
    render_pass::{RenderPassBuilder, RenderPassIntenal},
    render_pipeline::{RenderPipeline, RenderPipelineBuilder},
    shader::Shader,
    texture::FRAMEBUFFER,
};

pub struct RenderManager {
    pub window: Window,
    pub(crate) surface: Surface,
    pub(crate) device: Arc<Device>,
    pub(crate) queue: Arc<Queue>,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) size: PhysicalSize<u32>,
    pub(crate) passes: Registry<RenderPassIntenal>,
    pub(crate) pipelines: Registry<RenderPipeline>,
    pub(crate) shaders: Registry<Shader>,
    pub(crate) buffers: Registry<Buffer>,
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
            device: Arc::new(device),
            queue: Arc::new(queue),
            config,
            size: window_size,
            passes: Registry::new(),
            pipelines: Registry::new(),
            shaders: Registry::new(),
            buffers: Registry::new(),
        }
    }

    pub fn pipeline_builder<'a>(&'a mut self, label: Label<'a>) -> RenderPipelineBuilder<'a> {
        RenderPipelineBuilder::new(self, label)
    }

    pub fn pass_builder<'a>(&'a mut self, label: Label<'a>) -> RenderPassBuilder<'a> {
        RenderPassBuilder::new(self, label)
    }

    pub fn buffer_builder<'a, T: BufferContents>(
        &'a mut self,
        label: Label<'a>,
    ) -> BufferBuilder<'a, T> {
        BufferBuilder::new(self, label)
    }

    pub fn register_shader(&mut self, shader: &str, label: Label<'_>) -> Handle<Shader> {
        let module = self.device.create_shader_module(ShaderModuleDescriptor {
            label,
            source: ShaderSource::Wgsl(shader.into()),
        });

        self.shaders.add(Shader(module))
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
                    .get(*pipeline)
                    .expect("Invalid RenderPipelineHandle in a render pass");
                pass.set_pipeline(&pipeline.pipeline);

                for (i, vertex_buffer) in pipeline.vertex_buffers.iter().enumerate() {
                    pass.set_vertex_buffer(
                        i as u32,
                        self.buffers
                            .get(*vertex_buffer)
                            .expect("Invalid BufferHandle in a render pipeline")
                            .inner()
                            .slice(..),
                    )
                }

                pass.draw(0 .. 3, 0 .. 1);
            }
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}
