use std::{fs::OpenOptions, io::Read, path::Path, sync::Arc};

pub use wgpu::SurfaceError;
use wgpu::{
    Backends,
    CommandEncoder,
    CommandEncoderDescriptor,
    ComputePassDescriptor,
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
    RenderPassDepthStencilAttachment,
    RenderPassDescriptor,
    RequestAdapterOptions,
    ShaderModuleDescriptor,
    ShaderSource,
    Surface,
    SurfaceConfiguration,
    TextureUsages,
    TextureView,
    TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    bind_group::{BindGroup, BindGroupBuilder},
    buffer::{Buffer, BufferBuilder, BufferContents, BufferHandle},
    compute_pass::{ComputePass, ComputePassBuilder, ComputePassHandle},
    compute_pipeline::{ComputePipeline, ComputePipelineBuilder},
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
    passes: PassManager,
    render_passes: Registry<RenderPass>,
    compute_passes: Registry<ComputePass>,
    render_pipelines: Registry<RenderPipeline>,
    compute_pipelines: Registry<ComputePipeline>,
    shaders: Registry<Shader>,
    buffers: Registry<Buffer>,
    textures: Registry<Texture>,
    bind_groups: Registry<BindGroup>,
    samplers: Registry<TextureSampler>,
}

macro_rules! add_resource_methods {
    ($($adder: ident, $getter: ident, $field: ident, $type: ty),*) => {
        $(
            pub fn $adder(&mut self, resource: $type) -> Handle<$type> {
                self.$field.add(resource)
            }

            #[allow(unused)]
            pub(crate) fn $getter(&self, handle: Handle<$type>) -> Option<&$type> {
                self.$field.get(handle)
            }
        )*
    };
}
impl RenderManager {
    add_resource_methods! {
        add_render_pipeline, get_render_pipeline, render_pipelines, RenderPipeline,
        add_compute_pipeline, get_compute_pipeline, compute_pipelines, ComputePipeline,
        add_buffer, get_buffer, buffers, Buffer,
        add_texture, get_texture, textures, Texture,
        add_sampler, get_sampler, samplers, TextureSampler,
        add_bind_group, get_bind_group, bind_groups, BindGroup
    }

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
            passes: PassManager::new(),
            render_passes: Registry::new(),
            render_pipelines: Registry::new(),
            compute_passes: Registry::new(),
            compute_pipelines: Registry::new(),
            shaders: Registry::new(),
            buffers: Registry::new(),
            textures: Registry::new(),
            bind_groups: Registry::new(),
            samplers: Registry::new(),
        }
    }

    pub fn render_pipeline_builder<'a>(
        &'a mut self,
        label: Label<'a>,
    ) -> RenderPipelineBuilder<'a> {
        RenderPipelineBuilder::new(self, label)
    }

    pub fn compute_pipeline_builder<'a>(
        &'a mut self,
        label: Label<'a>,
    ) -> ComputePipelineBuilder<'a> {
        ComputePipelineBuilder::new(self, label)
    }

    pub fn render_pass_builder<'a>(&'a mut self, label: Label<'a>) -> RenderPassBuilder<'a> {
        RenderPassBuilder::new(self, label)
    }

    pub fn compute_pass_builder<'a>(&'a mut self, label: Label<'a>) -> ComputePassBuilder<'a> {
        ComputePassBuilder::new(self, label)
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

    pub fn add_render_pass(&mut self, pass: RenderPass) -> RenderPassHandle {
        let handle = self.render_passes.add(pass);
        self.passes.add_render_pass(handle);
        handle
    }

    pub fn add_compute_pass(&mut self, pass: ComputePass) -> ComputePassHandle {
        let handle = self.compute_passes.add(pass);
        self.passes.add_compute_pass(handle);
        handle
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

    pub(crate) fn get_shader(&self, handle: ShaderHandle) -> Option<&Shader> {
        self.shaders.get(handle)
    }

    pub fn reorder_pipelines(
        &mut self,
        pass: RenderPassHandle,
        pipelines: impl AsRef<[PipelineHandle]>,
    ) {
        if cfg!(debug_assertions) {
            for pipeline in pipelines.as_ref() {
                debug_assert!(
                    self.render_pipelines.get(*pipeline).is_some(),
                    "Invalid pipeline handle included in RenderManager::reorder_pipelines"
                )
            }
        }

        let pass = self
            .render_passes
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

        for pass in &self.passes {
            match pass {
                PassHandle::RenderPass(pass) =>
                    self.run_render_pass(pass, &mut command_encoder, &surface_view),
                PassHandle::ComputePass(pass) => self.run_compute_pass(pass, &mut command_encoder),
            }
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        surface_texture.present();

        Ok(())
    }

    fn run_compute_pass(&self, pass: ComputePassHandle, command_encoder: &mut CommandEncoder) {
        let pass_desc = self.compute_passes.get(pass).unwrap();
        let mut pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
            label: pass_desc.name.as_deref(),
        });

        for pipeline in &pass_desc.pipelines {
            let pipeline = self.compute_pipelines.get(*pipeline).unwrap();

            pass.set_pipeline(pipeline.inner());

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

            pass.dispatch_workgroups(
                pipeline.work_groups[0],
                pipeline.work_groups[1],
                pipeline.work_groups[2],
            )
        }
    }

    // Needed since we never read from depth_stencil_view
    // It's only used to keep the reference to the TextureView alive
    #[allow(unused_assignments)]
    fn run_render_pass(
        &self,
        pass: RenderPassHandle,
        command_encoder: &mut CommandEncoder,
        surface_view: &TextureView,
    ) {
        let mut views = Vec::new();
        let mut attachments = Vec::new();
        let pass_desc = self.render_passes.get(pass).unwrap();

        for (texture, _) in &pass_desc.color_attachments {
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

        for ((_, op), view) in pass_desc.color_attachments.iter().zip(views.iter()) {
            // TODO: add support for only enabling some attachements in a pass
            attachments.push(Some(RenderPassColorAttachment {
                view: if let Some(v) = view { v } else { surface_view },
                resolve_target: None,
                ops: *op,
            }));
        }

        let mut depth_stencil_view = None;
        let depth_stencil = if let Some(d) = &pass_desc.depth_attachments {
            depth_stencil_view = Some(
                self.textures
                    .get(d.texture)
                    .expect("Invalid TextureHandle in a render pass as a depth stencil attachment")
                    .get_view(),
            );
            Some(RenderPassDepthStencilAttachment {
                view: depth_stencil_view.as_ref().unwrap(),
                depth_ops: d.depth_op,
                stencil_ops: d.stencil_op,
            })
        } else {
            None
        };


        let mut pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: pass_desc.name.as_deref(),
            color_attachments: &attachments,
            depth_stencil_attachment: depth_stencil,
        });

        for pipeline in &pass_desc.pipelines {
            let pipeline = self
                .render_pipelines
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
                let idx_buffer = self
                    .buffers
                    .get(idx_buffer)
                    .expect("Invalid BufferHandle used as an index buffer in a render pipeline");
                let size = idx_buffer.len();
                pass.set_index_buffer(
                    idx_buffer.inner().slice(..),
                    match idx_buffer.inner().size() / size {
                        2 => IndexFormat::Uint16,
                        4 => IndexFormat::Uint32,
                        _ => panic!("Type of unsupported size used in an index buffer"),
                    },
                );

                let mut vertex_buffer_size = None;

                for (i, vertex_buffer) in pipeline.vertex_buffers.iter().enumerate() {
                    let buffer = self.buffers.get(*vertex_buffer).expect(
                        "Invalid BufferHandle used as a vertex buffer in a render pipeline",
                    );

                    if let Some(size) = vertex_buffer_size {
                        debug_assert!(
                            size == buffer.len(),
                            "Vertex buffers in render pipeline have different lengths. Found \
                             buffer with length {}, expected {size}.",
                            buffer.len()
                        )
                    } else {
                        vertex_buffer_size = Some(buffer.len());
                    }

                    pass.set_vertex_buffer(i as u32, buffer.inner().slice(..))
                }

                let max_vertex_buffer = pipeline.vertex_buffers.len();
                let mut instance_size = None;

                for (i, instance_buffer) in pipeline.instance_buffers.iter().enumerate() {
                    let buffer = self.buffers.get(*instance_buffer).expect(
                        "Invalid BufferHandle used as an instance buffer in a render pipeline",
                    );

                    if let Some(size) = instance_size {
                        debug_assert!(
                            buffer.len() as u32 == size,
                            "Instance buffers in render pipeline have different lengths, ensure \
                             all instance buffers have the same length",
                        )
                    } else {
                        instance_size = Some(buffer.len() as u32);
                    }

                    // We ensure that instance buffers come after vertex buffers
                    pass.set_vertex_buffer((i + max_vertex_buffer) as u32, buffer.inner().slice(..))
                }

                pass.draw_indexed(0 .. size as u32, 0, 0 .. instance_size.unwrap_or(1));
            } else {
                let mut vertex_buffer_size = None;

                for (i, vertex_buffer) in pipeline.vertex_buffers.iter().enumerate() {
                    let buffer = self
                        .buffers
                        .get(*vertex_buffer)
                        .expect("Invalid BufferHandle in a render pipeline");

                    if let Some(size) = vertex_buffer_size {
                        debug_assert!(
                            size == buffer.len(),
                            "Vertex buffers in render pipeline have different lengths. Found \
                             buffer with length {}, expected {size}.",
                            buffer.len()
                        )
                    } else {
                        vertex_buffer_size = Some(buffer.len());
                    }

                    pass.set_vertex_buffer(i as u32, buffer.inner().slice(..))
                }

                // If no vertex buffers were attached we just default to drawing one vertex
                // TODO: add a way to specify vertex count when no vertex buffers were attached
                pass.draw(0 .. vertex_buffer_size.unwrap_or(1) as u32, 0 .. 1);
            }
        }
    }
}

pub struct PassManager {
    render_passes: Vec<RenderPassHandle>,
    compute_passes: Vec<ComputePassHandle>,
    ordered_passes: Vec<(usize, PassType)>,
}

impl PassManager {
    pub(crate) fn new() -> PassManager {
        PassManager {
            render_passes: Vec::new(),
            compute_passes: Vec::new(),
            ordered_passes: Vec::new(),
        }
    }

    pub fn add_compute_pass(&mut self, handle: ComputePassHandle) {
        self.ordered_passes
            .push((self.compute_passes.len(), PassType::Compute));
        self.compute_passes.push(handle);
    }

    pub fn add_render_pass(&mut self, handle: RenderPassHandle) {
        self.ordered_passes
            .push((self.render_passes.len(), PassType::Render));
        self.render_passes.push(handle);
    }
}

impl<'a> IntoIterator for &'a PassManager {
    type IntoIter = PassIter<'a>;
    type Item = PassHandle;

    fn into_iter(self) -> Self::IntoIter {
        PassIter {
            render: &self.render_passes,
            compute: &self.compute_passes,
            ordered: &self.ordered_passes,
            curr: 0,
        }
    }
}

pub struct PassIter<'a> {
    render: &'a [RenderPassHandle],
    compute: &'a [ComputePassHandle],
    ordered: &'a [(usize, PassType)],
    curr: usize,
}

impl<'a> Iterator for PassIter<'a> {
    type Item = PassHandle;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self
            .ordered
            .get(self.curr)
            .and_then(|(i, kind)| match kind {
                PassType::Render => self.render.get(*i).copied().map(PassHandle::RenderPass),
                PassType::Compute => self.compute.get(*i).copied().map(PassHandle::ComputePass),
            });
        self.curr += 1;
        next
    }
}

pub enum PassType {
    Render,
    Compute,
}

pub enum PassHandle {
    RenderPass(RenderPassHandle),
    ComputePass(ComputePassHandle),
}
