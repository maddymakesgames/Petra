use wgpu::{
    ComputePipeline as RawComputePipeline,
    ComputePipelineDescriptor,
    Label,
    PipelineLayoutDescriptor,
};

use crate::{
    bind_group::BindGroupHandle,
    handle::Handle,
    manager::RenderManager,
    shader::ShaderHandle,
};

pub type ComputePipelineHandle = Handle<ComputePipeline>;

pub struct ComputePipeline {
    pipeline: RawComputePipeline,
    pub(crate) bind_groups: Vec<BindGroupHandle>,
    pub(crate) work_groups: [u32; 3],
}

impl ComputePipeline {
    pub fn inner(&self) -> &RawComputePipeline {
        &self.pipeline
    }
}

pub struct ComputePipelineBuilder<'a> {
    name: Label<'a>,
    manager: &'a mut RenderManager,
    bind_groups: Vec<BindGroupHandle>,
    shader: Option<ShaderHandle>,
    entry_point: Option<&'a str>,
    work_groups: Option<[u32; 3]>,
}

impl<'a> ComputePipelineBuilder<'a> {
    pub fn new(manager: &'a mut RenderManager, name: Label<'a>) -> ComputePipelineBuilder<'a> {
        ComputePipelineBuilder {
            name,
            manager,
            bind_groups: Vec::new(),
            shader: None,
            entry_point: None,
            work_groups: None,
        }
    }

    pub fn set_shader(mut self, handle: ShaderHandle, entry_point: &'a str) -> Self {
        self.shader = Some(handle);
        self.entry_point = Some(entry_point);

        self
    }

    pub fn add_bind_group(mut self, bind_group: BindGroupHandle) -> Self {
        self.bind_groups.push(bind_group);

        self
    }

    pub fn work_groups(mut self, work_groups: [u32; 3]) -> Self {
        self.work_groups = Some(work_groups);
        self
    }

    pub fn build(self) -> ComputePipelineHandle {
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

        self.manager.add_compute_pipeline(ComputePipeline {
            pipeline: self
                .manager
                .device
                .create_compute_pipeline(&ComputePipelineDescriptor {
                    label: self.name,
                    layout: Some(&pipeline_layout),
                    module: &self
                        .manager
                        .get_shader(
                            self.shader
                                .expect("No shader proveded in ComputePipelineBuilder"),
                        )
                        .expect("Invalid ShaderHandle passed to ComputePipelineBuilder")
                        .0,
                    entry_point: self.entry_point.unwrap(),
                }),
            bind_groups: self.bind_groups,
            work_groups: self
                .work_groups
                .expect("No work groups defined for a ComputePipelineBuilder"),
        })
    }
}
