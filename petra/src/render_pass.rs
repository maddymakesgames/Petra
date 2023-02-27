use wgpu::{Color, Label, LoadOp, Operations};

use crate::{
    handle::Handle,
    manager::RenderManager,
    render_pipeline::PipelineHandle,
    texture::{TextureHandle, FRAMEBUFFER},
};

pub type RenderPassHandle = Handle<RenderPass>;

pub struct RenderPass {
    pub name: Option<String>,
    pub color_attachments: Vec<(TextureHandle, Operations<Color>)>,
    pub depth_attachments: Option<DepthAttachment>,
    pub pipelines: Vec<PipelineHandle>,
}

impl RenderPass {
    pub fn reorder_pipelines(&mut self, pipeline: impl AsRef<[PipelineHandle]>) {
        self.pipelines = pipeline.as_ref().to_vec();
    }
}

pub struct DepthAttachment {
    pub texture: TextureHandle,
    pub depth_op: Option<Operations<f32>>,
    pub stencil_op: Option<Operations<u32>>,
}

pub struct RenderPassBuilder<'a> {
    manager: &'a mut RenderManager,
    color_attachments: Vec<(TextureHandle, Operations<Color>)>,
    depth_attachments: Option<DepthAttachment>,
    name: Label<'a>,
    pipelines: Vec<PipelineHandle>,
}

impl<'a> RenderPassBuilder<'a> {
    pub(crate) fn new(manager: &'a mut RenderManager, name: Label<'a>) -> RenderPassBuilder<'a> {
        RenderPassBuilder {
            manager,
            color_attachments: Vec::new(),
            depth_attachments: None,
            name,
            pipelines: Vec::new(),
        }
    }

    pub fn add_color_attachment(
        mut self,
        texture: TextureHandle,
        clear_color: Option<Color>,
        store: bool,
    ) -> RenderPassBuilder<'a> {
        self.color_attachments.push((texture, Operations {
            load: clear_color.map(LoadOp::Clear).unwrap_or(LoadOp::Load),
            store,
        }));
        self
    }

    pub fn add_pipeline(mut self, pipeline: PipelineHandle) -> RenderPassBuilder<'a> {
        self.pipelines.push(pipeline);
        self
    }

    pub fn add_depth_stencil_attachment(
        mut self,
        texture: TextureHandle,
        depth: Option<(Option<f32>, bool)>,
        stencil: Option<(Option<u32>, bool)>,
    ) -> Self {
        self.depth_attachments = Some(DepthAttachment {
            texture,
            depth_op: depth.map(|(clear, store)| Operations {
                load: clear.map(LoadOp::Clear).unwrap_or(LoadOp::Load),
                store,
            }),
            stencil_op: stencil.map(|(clear, store)| Operations {
                load: clear.map(LoadOp::Clear).unwrap_or(LoadOp::Load),
                store,
            }),
        });
        self
    }

    pub fn build(mut self) -> RenderPassHandle {
        // Assume that if no color attachments were added
        // then we want to render just to the framebuffer
        if self.color_attachments.is_empty() {
            self.color_attachments.push((FRAMEBUFFER, Operations {
                load: LoadOp::Load,
                store: true,
            }));
        }


        self.manager.add_render_pass(RenderPass {
            name: self.name.map(str::to_owned),
            color_attachments: self.color_attachments,
            depth_attachments: self.depth_attachments,
            pipelines: self.pipelines,
        })
    }
}
