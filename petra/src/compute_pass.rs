use wgpu::Label;

use crate::{compute_pipeline::ComputePipelineHandle, handle::Handle, manager::RenderManager};

pub type ComputePassHandle = Handle<ComputePass>;

pub struct ComputePass {
    pub(crate) name: Option<String>,
    pub(crate) pipelines: Vec<ComputePipelineHandle>,
}

pub struct ComputePassBuilder<'a> {
    name: Label<'a>,
    manager: &'a mut RenderManager,
    pipelines: Vec<ComputePipelineHandle>,
}

impl<'a> ComputePassBuilder<'a> {
    pub fn new(manager: &'a mut RenderManager, name: Label<'a>) -> ComputePassBuilder<'a> {
        ComputePassBuilder {
            name,
            manager,
            pipelines: Vec::new(),
        }
    }

    pub fn add_pipeline(mut self, pipeline: ComputePipelineHandle) -> Self {
        self.pipelines.push(pipeline);
        self
    }

    pub fn build(self) -> ComputePassHandle {
        self.manager.add_compute_pass(ComputePass {
            name: self.name.map(|s| s.to_owned()),
            pipelines: self.pipelines,
        })
    }
}
