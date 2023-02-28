use wgpu::{
    CompareFunction,
    DepthBiasState,
    DepthStencilState,
    FragmentState,
    Label,
    MultisampleState,
    PipelineLayoutDescriptor,
    PrimitiveState,
    RenderPipeline as RawRenderPipeline,
    RenderPipelineDescriptor,
    StencilState,
    VertexState,
};
pub use wgpu::{Face, FrontFace, PolygonMode, PrimitiveTopology};

use crate::{
    bind_group::BindGroupHandle,
    buffer::BufferHandle,
    handle::Handle,
    manager::RenderManager,
    shader::ShaderHandle,
    texture::TextureContents,
};

pub type PipelineHandle = Handle<RenderPipeline>;

pub struct RenderPipeline {
    pub(crate) pipeline: RawRenderPipeline,
    pub(crate) vertex_buffers: Vec<BufferHandle>,
    pub(crate) instance_buffers: Vec<BufferHandle>,
    pub(crate) bind_groups: Vec<BindGroupHandle>,
    pub(crate) index_buffers: Option<Handle<crate::buffer::Buffer>>,
}

pub struct RenderPipelineBuilder<'a> {
    manager: &'a mut RenderManager,
    name: Label<'a>,
    vertex_shader: Option<(&'a str, ShaderHandle)>,
    fragment_shader: Option<(&'a str, ShaderHandle)>,
    topology: Option<PrimitiveTopology>,
    front_face: Option<FrontFace>,
    culling: Option<Face>,
    polygon_mode: PolygonMode,
    vertex_buffers: Vec<BufferHandle>,
    index_buffer: Option<BufferHandle>,
    instance_buffers: Vec<BufferHandle>,
    bind_groups: Vec<BindGroupHandle>,
    depth_stencil: Option<DepthStencilState>,
    unclipped_depth: bool,
    conservative: bool,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub(crate) fn new(manager: &'a mut RenderManager, name: Label<'a>) -> Self {
        RenderPipelineBuilder {
            manager,
            name,
            vertex_shader: None,
            fragment_shader: None,
            topology: None,
            front_face: None,
            culling: None,
            polygon_mode: PolygonMode::Fill,
            vertex_buffers: Vec::new(),
            index_buffer: None,
            instance_buffers: Vec::new(),
            bind_groups: Vec::new(),
            depth_stencil: None,
            unclipped_depth: false,
            conservative: false,
        }
    }

    pub fn vertex_shader(mut self, shader: ShaderHandle, entry_point: &'a str) -> Self {
        self.vertex_shader = Some((entry_point, shader));
        self
    }

    pub fn fragment_shader(mut self, shader: ShaderHandle, entry_point: &'a str) -> Self {
        self.fragment_shader = Some((entry_point, shader));
        self
    }

    pub fn topology(mut self, topology: PrimitiveTopology) -> Self {
        self.topology = Some(topology);
        self
    }

    pub fn polygon_mode(mut self, polygon_mode: PolygonMode) -> Self {
        self.polygon_mode = polygon_mode;
        self
    }

    pub fn front_face(mut self, front_face: FrontFace) -> Self {
        self.front_face = Some(front_face);
        self
    }

    pub fn culling(mut self, culling_face: Face) -> Self {
        self.culling = Some(culling_face);
        self
    }

    pub fn add_vertex_buffer(mut self, buffer: BufferHandle) -> Self {
        self.vertex_buffers.push(buffer);
        self
    }

    pub fn add_instance_buffer(mut self, buffer: BufferHandle) -> Self {
        self.instance_buffers.push(buffer);
        self
    }

    pub fn add_bind_group(mut self, bind_group: BindGroupHandle) -> Self {
        self.bind_groups.push(bind_group);
        self
    }

    pub fn add_index_buffer(mut self, buffer: BufferHandle) -> Self {
        self.index_buffer = Some(buffer);
        self
    }

    pub fn depth_stencil<C: TextureContents>(
        mut self,
        write_enabled: bool,
        compare: CompareFunction,
        stencil: StencilState,
        bias: DepthBiasState,
    ) -> Self {
        self.depth_stencil = Some(DepthStencilState {
            format: C::FORMAT,
            depth_write_enabled: write_enabled,
            depth_compare: compare,
            stencil,
            bias,
        });

        self
    }

    pub fn unclipped_depth(mut self) -> Self {
        self.unclipped_depth = true;
        self
    }

    pub fn conservative_rasterization(mut self) -> Self {
        self.conservative = true;
        self
    }

    pub fn build(self) -> PipelineHandle {
        let mut bind_group_layouts = Vec::with_capacity(self.bind_groups.len());

        for group in &self.bind_groups {
            let group = self
                .manager
                .get_bind_group(*group)
                .expect("Invalid BindGroupHandle passed to RenderPipelineBuilder");
            bind_group_layouts.push(group.layout());
        }

        let pipeline_layout =
            self.manager
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: self.name,
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let (vert_entry_point, vert_shader) = self
            .vertex_shader
            .expect("Vertex Shader not defined when building a render pipeline");

        let formats = &[Some(self.manager.config.format.into())];
        let fragment_state = if let Some((entry_point, handle)) = self.fragment_shader {
            let module = &self
                .manager
                .get_shader(handle)
                .expect("Invalid Shader Handle passed as a fragment shader")
                .0;

            Some(FragmentState {
                module,
                entry_point,
                targets: formats,
            })
        } else {
            None
        };

        let vert_shader = &self
            .manager
            .get_shader(vert_shader)
            .expect("Invalid Shader Handle passed as a vertex shader")
            .0;

        let mut vertex_buffers = Vec::with_capacity(self.vertex_buffers.len());

        for handle in &self.vertex_buffers {
            let buffer = self
                .manager
                .get_buffer(*handle)
                .expect("Invalid Buffer Handle passed as a vertex buffer");

            vertex_buffers.push(buffer.vertex_format().unwrap_or_else(|| {
                panic!(
                    "Attempted to attach buffer {:?} to pipeline {:?} as a vertex buffer, but the \
                     buffer cannot be used as a vertex buffer",
                    buffer.name(),
                    self.name
                )
            }));
        }

        for handle in &self.instance_buffers {
            let buffer = self
                .manager
                .get_buffer(*handle)
                .expect("Invalid Buffer Handle passed as an instance buffer");

            vertex_buffers.push(buffer.vertex_format().unwrap_or_else(|| {
                panic!(
                    "Attempted to attach buffer {:?} to pipeline {:?} as an instance buffer, but \
                     the buffer cannot be used as an instance buffer",
                    buffer.name(),
                    self.name
                )
            }));
        }

        let pipeline = self
            .manager
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: self.name,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: vert_shader,
                    entry_point: vert_entry_point,
                    buffers: &vertex_buffers,
                },
                primitive: PrimitiveState {
                    topology: self
                        .topology
                        .expect("Topology not defined when building render pipeline"),
                    strip_index_format: if self.topology.unwrap().is_strip()
                        && self.index_buffer.is_some()
                    {
                        let buffer = self.index_buffer.unwrap();
                        self.manager.get_buffer(buffer).unwrap().index_format()
                    } else {
                        None
                    },
                    front_face: self
                        .front_face
                        .expect("Front face not defined when building render pipeline"),
                    cull_mode: self.culling,
                    unclipped_depth: self.unclipped_depth,
                    polygon_mode: self.polygon_mode,
                    conservative: self.conservative,
                },
                depth_stencil: self.depth_stencil,
                multisample: MultisampleState::default(),
                fragment: fragment_state,
                multiview: None,
            });

        let pipeline = RenderPipeline {
            pipeline,
            vertex_buffers: self.vertex_buffers,
            instance_buffers: self.instance_buffers,
            index_buffers: self.index_buffer,
            bind_groups: self.bind_groups,
        };

        self.manager.add_render_pipeline(pipeline)
    }
}
