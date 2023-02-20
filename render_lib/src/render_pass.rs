use wgpu::{Color, Label, LoadOp, Operations};

use crate::{
    manager::RenderManager,
    render_pipeline::RenderPipelineHandle,
    texture::TextureHandle,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderPassHandle(pub(crate) usize);

pub(crate) struct RenderPassIntenal {
    pub name: Option<String>,
    pub attachments: Vec<(TextureHandle, Operations<Color>)>,
    pub pipelines: Vec<RenderPipelineHandle>,
}

pub struct RenderPassBuilder<'a> {
    manager: &'a mut RenderManager,
    attachments: Vec<(TextureHandle, Operations<Color>)>,
    name: Label<'a>,
    pipelines: Vec<RenderPipelineHandle>,
}

impl<'a> RenderPassBuilder<'a> {
    pub(crate) fn new(manager: &'a mut RenderManager, name: Label<'a>) -> RenderPassBuilder<'a> {
        RenderPassBuilder {
            manager,
            attachments: Vec::new(),
            name,
            pipelines: Vec::new(),
        }
    }

    pub fn add_attachment(
        mut self,
        texture: TextureHandle,
        clear_color: Option<Color>,
        store: bool,
    ) -> RenderPassBuilder<'a> {
        self.attachments.push((texture, Operations {
            load: if let Some(color) = clear_color {
                LoadOp::Clear(color)
            } else {
                LoadOp::Load
            },
            store,
        }));
        self
    }

    pub fn add_pipeline(mut self, pipeline: RenderPipelineHandle) -> RenderPassBuilder<'a> {
        self.pipelines.push(pipeline);
        self
    }

    pub fn build(mut self) -> RenderPassHandle {
        if self.attachments.is_empty() {
            self.attachments
                .push((TextureHandle::FRAMEBUFFER, Operations {
                    load: LoadOp::Load,
                    store: true,
                }));
        }

        let id = self.manager.passes.len();

        self.manager.passes.push(RenderPassIntenal {
            name: self.name.map(str::to_owned),
            attachments: self.attachments,
            pipelines: self.pipelines,
        });

        RenderPassHandle(id)
    }
}
