use std::num::NonZeroU64;

use wgpu::{
    BindGroup as RawBindGroup,
    BindGroupDescriptor,
    BindGroupEntry,
    BindGroupLayout,
    BindGroupLayoutDescriptor,
    BindGroupLayoutEntry,
    BindingResource,
    BindingType,
    BufferBindingType,
    Device,
    Label,
    SamplerBindingType,
    ShaderStages,
    StorageTextureAccess,
    TextureSampleType,
    TextureViewDimension,
};

use crate::{
    buffer::{Buffer, BufferContents, BufferHandle},
    handle::{Handle, Registry},
    manager::RenderManager,
    texture::{Texture, TextureHandle},
};

pub type BindGroupHandle = Handle<BindGroup>;

pub struct BindGroup {
    name: Option<String>,
    layout: BindGroupLayout,
    bind_group: RawBindGroup,
    buffers: Vec<(u32, BufferHandle)>,
    textures: Vec<(u32, TextureHandle)>,
    samplers: Vec<(u32, ())>,
}

impl BindGroup {
    fn new(
        name: Label<'_>,
        layout: BindGroupLayout,
        buffers: Vec<(u32, BufferHandle)>,
        textures: Vec<(u32, TextureHandle)>,
        samplers: Vec<(u32, ())>,
        manager: &mut RenderManager,
    ) -> Self {
        let mut entries = Vec::new();
        let mut views = Vec::new();

        for (binding, buffer) in &buffers {
            let buffer = manager
                .buffers
                .get(*buffer)
                .expect("Invalid BufferHandle passed to BindGroupBuilder");

            entries.push(BindGroupEntry {
                binding: *binding,
                resource: BindingResource::Buffer(buffer.inner().as_entire_buffer_binding()),
            })
        }

        for (binding, texture) in &textures {
            let texture = manager
                .textures
                .get(*texture)
                .expect("Invalid TextureHandle passed to BindGroupBuilder");

            let view = texture.get_view();

            views.push((*binding, view));
        }

        for (binding, view) in &views {
            entries.push(BindGroupEntry {
                binding: *binding,
                resource: BindingResource::TextureView(view),
            })
        }

        for (binding, _sampler) in &samplers {
            entries.push(BindGroupEntry {
                binding: *binding,
                resource: BindingResource::Sampler(todo!("I need to define samplers first lol")),
            })
        }

        let bind_group = manager.device.create_bind_group(&BindGroupDescriptor {
            label: name,
            layout: &layout,
            entries: &entries,
        });

        Self {
            name: name.map(|s| s.to_owned()),
            bind_group,
            layout,
            buffers,
            textures,
            samplers,
        }
    }

    pub(crate) fn inner(&self) -> &RawBindGroup {
        &self.bind_group
    }

    pub(crate) fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    pub(crate) fn depends_texture(&self, texture: TextureHandle) -> bool {
        self.textures.iter().any(|(_, h)| *h == texture)
    }

    pub(crate) fn depends_buffer(&self, buffer: BufferHandle) -> bool {
        self.buffers.iter().any(|(_, h)| *h == buffer)
    }

    /// Recreates the BindGroup in case some of the buffers or textures have been recreated
    pub(crate) fn recreate(
        &mut self,
        device: &Device,
        buffers: &Registry<Buffer>,
        textures: &Registry<Texture>,
    ) {
        let mut entries = Vec::new();
        let mut views = Vec::new();

        for (binding, buffer) in &self.buffers {
            let buffer = buffers
                .get(*buffer)
                .expect("Invalid BufferHandle passed to BindGroupBuilder");

            entries.push(BindGroupEntry {
                binding: *binding,
                resource: BindingResource::Buffer(buffer.inner().as_entire_buffer_binding()),
            })
        }

        for (binding, texture) in &self.textures {
            let texture = textures
                .get(*texture)
                .expect("Invalid TextureHandle passed to BindGroupBuilder");

            let view = texture.get_view();

            views.push((*binding, view));
        }

        for (binding, view) in &views {
            entries.push(BindGroupEntry {
                binding: *binding,
                resource: BindingResource::TextureView(view),
            })
        }

        for (binding, _sampler) in &self.samplers {
            entries.push(BindGroupEntry {
                binding: *binding,
                resource: BindingResource::Sampler(todo!("I need to define samplers first lol")),
            })
        }

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: self.name.as_deref(),
            layout: &self.layout,
            entries: &entries,
        });

        self.bind_group = bind_group;
    }
}


pub struct BindGroupBuilder<'a> {
    name: Label<'a>,
    manager: &'a mut RenderManager,
    entries: Vec<BindGroupLayoutEntry>,
    buffers: Vec<(u32, BufferHandle)>,
    textures: Vec<(u32, TextureHandle)>,
    samplers: Vec<(u32, ())>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new(manager: &'a mut RenderManager, label: Label<'a>) -> BindGroupBuilder<'a> {
        BindGroupBuilder {
            name: label,
            manager,
            entries: Vec::new(),
            textures: Vec::new(),
            samplers: Vec::new(),
            buffers: Vec::new(),
        }
    }

    pub fn bind_uniform_buffer<T: BufferContents>(
        mut self,
        binding: u32,
        visibility: ShaderStages,
        buffer: BufferHandle,
    ) -> Self {
        debug_assert!(
            std::mem::size_of::<T>() as u64 % wgpu::MAP_ALIGNMENT == 0,
            "Data accessed by shaders must have an alignment of 8"
        );
        self.entries.push(BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(std::mem::size_of::<T>() as u64),
            },
            count: None,
        });

        self.buffers.push((binding, buffer));

        self
    }

    pub fn bind_storage_buffer<T: BufferContents>(
        mut self,
        binding: u32,
        visibility: ShaderStages,
        read_only: bool,
        num_elements: Option<u64>,
        buffer: BufferHandle,
    ) -> Self {
        debug_assert!(
            std::mem::size_of::<T>() as u64 % wgpu::MAP_ALIGNMENT == 0,
            "Data accessed by shaders must have an alignment of 8"
        );
        self.entries.push(BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: num_elements
                    .and_then(|size| NonZeroU64::new(size * std::mem::size_of::<T>() as u64)),
            },
            count: None,
        });

        self.buffers.push((binding, buffer));

        self
    }

    pub fn bind_texture(
        mut self,
        binding: u32,
        visibility: ShaderStages,
        sample_type: TextureSampleType,
        view_dimension: TextureViewDimension,
        multisampled: bool,
        texture: TextureHandle,
    ) -> Self {
        self.entries.push(BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Texture {
                sample_type,
                view_dimension,
                multisampled,
            },
            count: None,
        });

        self.textures.push((binding, texture));

        self
    }

    pub fn bind_storage_texture(
        mut self,
        binding: u32,
        visibility: ShaderStages,
        access: StorageTextureAccess,
        view_dimension: TextureViewDimension,
        texture: TextureHandle,
    ) -> Self {
        let format = self
            .manager
            .textures
            .get(texture)
            .expect("Invalid texture handle passed to bind_storage_texture")
            .format();

        self.entries.push(BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::StorageTexture {
                access,
                format,
                view_dimension,
            },
            count: None,
        });

        self.textures.push((binding, texture));

        self
    }

    pub fn bind_sampler(
        mut self,
        binding: u32,
        visibility: ShaderStages,
        kind: SamplerBindingType,
    ) -> Self {
        self.entries.push(BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Sampler(kind),
            count: None,
        });

        self
    }

    pub fn build(self) -> BindGroupHandle {
        let layout = self
            .manager
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: self.name,
                entries: &self.entries,
            });

        let group = BindGroup::new(
            self.name,
            layout,
            self.buffers,
            self.textures,
            self.samplers,
            self.manager,
        );
        self.manager.bind_groups.add(group)
    }
}
