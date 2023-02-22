pub use wgpu::{Face, FrontFace, PolygonMode, PrimitiveTopology};
use wgpu::{
    FragmentState,
    Label,
    MultisampleState,
    PipelineLayoutDescriptor,
    PrimitiveState,
    RenderPipelineDescriptor,
    VertexState,
};

use crate::{handle::Handle, manager::RenderManager, shader::Shader};

pub type PipelineHandle = Handle<RenderPipeline>;

pub struct RenderPipeline;

pub struct RenderPipelineBuilder<'a> {
    manager: &'a mut RenderManager,
    name: Label<'a>,
    vertex_shader: Option<(&'a str, Handle<Shader>)>,
    fragment_shader: Option<(&'a str, Handle<Shader>)>,
    topology: Option<PrimitiveTopology>,
    front_face: Option<FrontFace>,
    culling: Option<Face>,
    polygon_mode: PolygonMode,
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
        }
    }

    pub fn vertex_shader(mut self, shader: Handle<Shader>, entry_point: &'a str) -> Self {
        self.vertex_shader = Some((entry_point, shader));
        self
    }

    pub fn fragment_shader(mut self, shader: Handle<Shader>, entry_point: &'a str) -> Self {
        self.fragment_shader = Some((entry_point, shader));
        self
    }

    pub fn topology(mut self, topology: PrimitiveTopology) -> Self {
        self.topology = Some(topology);
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

    pub fn build(self) -> PipelineHandle {
        let pipeline_layout =
            self.manager
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: self.name,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let (vert_entry_point, vert_shader) = self
            .vertex_shader
            .expect("Vertex Shader not defined when building a render pipeline");

        let formats = &[Some(self.manager.config.format.into())];
        let fragment_state = if let Some((entry_point, handle)) = self.fragment_shader {
            let module = &self
                .manager
                .shaders
                .get(handle.index())
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
            .shaders
            .get(vert_shader.index())
            .expect("Invalid Shader Handle passed as a vertex shader")
            .0;

        let pipeline = self
            .manager
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: self.name,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: vert_shader,
                    entry_point: vert_entry_point,
                    buffers: &[],
                },
                primitive: PrimitiveState {
                    topology: self
                        .topology
                        .expect("Topology not defined when building render pipeline"),
                    strip_index_format: None,
                    front_face: self
                        .front_face
                        .expect("Front face not defined when building render pipeline"),
                    cull_mode: self.culling,
                    unclipped_depth: false,
                    polygon_mode: self.polygon_mode,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                fragment: fragment_state,
                multiview: None,
            });

        let pipeline_id = self.manager.pipelines.len();
        self.manager.pipelines.push(pipeline);

        Handle::new(pipeline_id)
    }
}
