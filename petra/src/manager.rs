use std::{fs::OpenOptions, io::Read, path::Path, sync::Arc};

pub use wgpu::SurfaceError;
use wgpu::{
    Backends,
    CommandEncoderDescriptor,
    Device,
    DeviceDescriptor,
    Dx12Compiler,
    Features,
    IndexFormat,
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
    bind_group::{BindGroup, BindGroupBuilder},
    buffer::{Buffer, BufferBuilder, BufferContents, BufferHandle},
    handle::{Handle, Registry},
    render_pass::{RenderPass, RenderPassBuilder, RenderPassHandle},
    render_pipeline::{PipelineHandle, RenderPipeline, RenderPipelineBuilder},
    sampler::{TextureSampler, TextureSamplerBuilder},
    shader::{Shader, ShaderHandle},
    texture::{Texture, TextureBuilder, TextureContents, FRAMEBUFFER},
};

pub struct RenderManager {
    pub window: Window,
    pub(crate) surface: Surface,
    pub(crate) device: Arc<Device>,
    pub(crate) queue: Arc<Queue>,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) size: PhysicalSize<u32>,
    pub(crate) passes: Registry<RenderPass>,
    pub(crate) pipelines: Registry<RenderPipeline>,
    pub(crate) shaders: Registry<Shader>,
    pub(crate) buffers: Registry<Buffer>,
    pub(crate) textures: Registry<Texture>,
    pub(crate) bind_groups: Registry<BindGroup>,
    pub(crate) samplers: Registry<TextureSampler>,
    ordered_passes: Vec<RenderPassHandle>,
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
            textures: Registry::new(),
            bind_groups: Registry::new(),
            samplers: Registry::new(),
            ordered_passes: Vec::new(),
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

    pub fn texture_builder<'a, T: TextureContents>(
        &'a mut self,
        label: Label<'a>,
    ) -> TextureBuilder<'a, T> {
        TextureBuilder::new(self, label)
    }

    pub fn bind_group_builder<'a>(&'a mut self, label: Label<'a>) -> BindGroupBuilder<'a> {
        BindGroupBuilder::new(self, label)
    }

    pub fn texture_sampler_builder<'a>(
        &'a mut self,
        label: Label<'a>,
    ) -> TextureSamplerBuilder<'a> {
        TextureSamplerBuilder::new(self, label)
    }

    pub fn write_to_buffer<T: BufferContents>(&mut self, buffer: BufferHandle, data: &[T]) {
        let raw_buffer = self
            .buffers
            .get_mut(buffer)
            .expect("Invalid buffer handle passed to write_to_buffer");

        // If the buffer had to be resized that means the old buffer was destroyed
        // We need to recreate any bind groups that depend on it
        if raw_buffer.write_data(data) {
            for bind_group in (&mut self.bind_groups)
                .into_iter()
                .filter(|b| b.depends_buffer(buffer))
            {
                bind_group.recreate(&self.device, &self.buffers, &self.textures, &self.samplers)
            }
        }
    }

    pub fn register_shader(&mut self, shader: &str, label: Label<'_>) -> ShaderHandle {
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
    ) -> std::io::Result<ShaderHandle> {
        let mut file = OpenOptions::new().read(true).open(shader)?;
        let mut buf = String::with_capacity(file.metadata().map(|m| m.len() as usize).unwrap_or(0));
        file.read_to_string(&mut buf)?;
        Ok(self.register_shader(&buf, label))
    }

    pub fn reorder_passes(&mut self, passes: impl AsRef<[RenderPassHandle]>) {
        if cfg!(debug_assertions) {
            for pass in passes.as_ref() {
                debug_assert!(
                    self.passes.get(*pass).is_some(),
                    "Invalid pass handle included in RenderManager::reorder_passes"
                )
            }
        }

        self.ordered_passes = passes.as_ref().to_vec();
    }

    pub fn reorder_pipelines(
        &mut self,
        pass: RenderPassHandle,
        pipelines: impl AsRef<[PipelineHandle]>,
    ) {
        if cfg!(debug_assertions) {
            for pipeline in pipelines.as_ref() {
                debug_assert!(
                    self.pipelines.get(*pipeline).is_some(),
                    "Invalid pipeline handle included in RenderManager::reorder_pipelines"
                )
            }
        }

        let pass = self
            .passes
            .get_mut(pass)
            .expect("Invalid RenderPassHandle in reorder_pipelines");

        pass.reorder_pipelines(pipelines);
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);

        let mut updated_textures = Vec::new();

        for (i, texture) in (&mut self.textures).into_iter().enumerate() {
            if texture.on_resize(&self.config) {
                updated_textures.push(i);
            }
        }
        for texture in updated_textures {
            for group in (&mut self.bind_groups)
                .into_iter()
                .filter(|g| g.depends_texture(Handle::new(texture)))
            {
                group.recreate(&self.device, &self.buffers, &self.textures, &self.samplers);
            }
        }
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
            let mut views = Vec::new();
            let mut attachments = Vec::new();

            for (texture, _) in &pass_desc.attachments {
                if *texture == FRAMEBUFFER {
                    views.push(None);
                } else {
                    views.push(Some(
                        self.textures
                            .get(*texture)
                            .expect("Invalid TextureHandle found in a render pass")
                            .get_view(),
                    ))
                };
            }

            for ((_, op), view) in pass_desc.attachments.iter().zip(views.iter()) {
                // TODO: add support for only enabling some attachements in a pass
                attachments.push(Some(RenderPassColorAttachment {
                    view: if let Some(v) = view { v } else { &surface_view },
                    resolve_target: None,
                    ops: *op,
                }));
            }

            let mut pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: pass_desc.name.as_deref(),
                color_attachments: &attachments,
                depth_stencil_attachment: None,
            });

            for pipeline in &pass_desc.pipelines {
                let pipeline = self
                    .pipelines
                    .get(*pipeline)
                    .expect("Invalid RenderPipelineHandle in a render pass");
                pass.set_pipeline(&pipeline.pipeline);

                for (i, bind_group) in pipeline.bind_groups.iter().enumerate() {
                    pass.set_bind_group(
                        i as u32,
                        self.bind_groups
                            .get(*bind_group)
                            .expect("Invalid BindGroupHandle in a render pipeline")
                            .inner(),
                        &[],
                    );
                }

                if let Some(idx_buffer) = pipeline.index_buffers {
                    let idx_buffer = self.buffers.get(idx_buffer).expect(
                        "Invalid BufferHandle used as an index buffer in a render pipeline",
                    );
                    let size = idx_buffer.len();
                    pass.set_index_buffer(
                        idx_buffer.inner().slice(..),
                        match idx_buffer.inner().size() / size {
                            2 => IndexFormat::Uint16,
                            4 => IndexFormat::Uint32,
                            _ => panic!("Type of unsupported size used in an index buffer"),
                        },
                    );

                    for (i, vertex_buffer) in pipeline.vertex_buffers.iter().enumerate() {
                        pass.set_vertex_buffer(
                            i as u32,
                            self.buffers
                                .get(*vertex_buffer)
                                .expect(
                                    "Invalid BufferHandle used as a vertex buffer in a render \
                                     pipeline",
                                )
                                .inner()
                                .slice(..),
                        )
                    }

                    pass.draw_indexed(0 .. size as u32, 0, 0 .. 1);
                } else {
                    let mut min_size = u64::MAX;

                    for (i, vertex_buffer) in pipeline.vertex_buffers.iter().enumerate() {
                        let buffer = self
                            .buffers
                            .get(*vertex_buffer)
                            .expect("Invalid BufferHandle in a render pipeline");

                        if buffer.len() < min_size {
                            min_size = buffer.len();
                        }

                        pass.set_vertex_buffer(i as u32, buffer.inner().slice(..))
                    }

                    pass.draw(0 .. min_size as u32, 0 .. 1);
                }
            }
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}
