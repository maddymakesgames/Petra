use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    sync::Arc,
};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer as RawBuffer,
    BufferDescriptor,
    BufferUsages,
    Device,
    Label,
    Queue,
    VertexBufferLayout,
    VertexStepMode,
};

use crate::{
    handle::Handle,
    manager::RenderManager,
    vertex::{vertex_format, Vertex},
};

pub type BufferHandle = Handle<Buffer>;

/// A marker trait that represents anything that can be safely sent to the gpu in a buffer
///
/// # Repr note
/// The gpu assumes structs have `#[repr(C, align(8))]`<br>
/// Statisfying this usually involves adding padding fields to satisfy the conditions for [bytemuck::Pod]
pub trait BufferContents: Any + Clone + Copy + Pod + Zeroable + Sized {}

impl<T: Any + Clone + Copy + Pod + Zeroable + Sized> BufferContents for T {}

pub struct Buffer {
    name: Option<String>,
    buffer: RawBuffer,
    type_id: TypeId,
    element_size: u64,
    queue: Arc<Queue>,
    device: Arc<Device>,
    vertex_format: Option<VertexBufferLayout<'static>>,
}

impl Buffer {
    fn new<T: BufferContents>(
        manager: &mut RenderManager,
        label: Label<'_>,
        size: u64,
        usage: BufferUsages,
        vertex_format: Option<VertexBufferLayout<'static>>,
    ) -> Buffer {
        debug_assert!(
            size % wgpu::MAP_ALIGNMENT == 0,
            "Data accessed by shaders must have an alignment of 8"
        );
        let raw = manager.device.create_buffer(&BufferDescriptor {
            label,
            size,
            usage,
            mapped_at_creation: false,
        });

        Buffer {
            buffer: raw,
            type_id: TypeId::of::<T>(),
            element_size: std::mem::size_of::<T>() as u64,
            queue: manager.queue.clone(),
            device: manager.device.clone(),
            name: label.map(|s| s.to_owned()),
            vertex_format,
        }
    }

    fn new_init<T: BufferContents>(
        manager: &mut RenderManager,
        label: Label<'_>,
        usage: BufferUsages,
        data: Vec<T>,
        vertex_format: Option<VertexBufferLayout<'static>>,
    ) -> Buffer {
        let raw = manager.device.create_buffer_init(&BufferInitDescriptor {
            label,
            usage,
            contents: bytemuck::cast_slice(&data),
        });

        Buffer {
            buffer: raw,
            type_id: TypeId::of::<T>(),
            element_size: std::mem::size_of::<T>() as u64,
            queue: manager.queue.clone(),
            device: manager.device.clone(),
            name: label.map(|s| s.to_owned()),
            vertex_format,
        }
    }

    pub fn write_data<T: BufferContents>(&mut self, data: &[T]) -> bool {
        if TypeId::of::<T>() != self.type_id {
            panic!(
                "Attempted to write to buffer with a different type than it was initialized with"
            );
        }
        let byte_slice = bytemuck::cast_slice(data);

        if byte_slice.len() as u64 > self.buffer.size() {
            let usage = self.buffer.usage();
            let old_buf = std::mem::replace(
                &mut self.buffer,
                self.device.create_buffer_init(&BufferInitDescriptor {
                    label: self.name.as_deref(),
                    contents: byte_slice,
                    usage,
                }),
            );

            old_buf.destroy();
            true
        } else {
            self.queue.write_buffer(&self.buffer, 0, byte_slice);
            false
        }
    }

    pub(crate) fn inner(&self) -> &RawBuffer {
        &self.buffer
    }

    pub(crate) fn len(&self) -> u64 {
        self.buffer.size() / self.element_size
    }

    pub(crate) fn vertex_format(&self) -> Option<VertexBufferLayout<'static>> {
        self.vertex_format.clone()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

pub struct BufferBuilder<'a, T: BufferContents> {
    usages: BufferUsages,
    label: Label<'a>,
    manager: &'a mut RenderManager,
    vertex_format: Option<VertexBufferLayout<'static>>,
    __buffer_type: PhantomData<T>,
}

macro_rules! usages {
    ($($func_name: ident, $usage: ident),*) => {
        $(
            pub fn $func_name(mut self) -> Self {
                self.usages |= BufferUsages::$usage;
                self
            }
        )*
    };
}

impl<'a, T: BufferContents> BufferBuilder<'a, T> {
    usages! {
        map_read, MAP_READ,
        map_write, MAP_WRITE,
        copy_src, COPY_SRC,
        copy_dst, COPY_DST,
        storage, STORAGE,
        uniform, UNIFORM,
        indirect, INDIRECT
    }

    pub(crate) fn new(manager: &'a mut RenderManager, label: Label<'a>) -> BufferBuilder<'a, T> {
        BufferBuilder {
            usages: BufferUsages::empty(),
            label,
            manager,
            vertex_format: None,
            __buffer_type: PhantomData,
        }
    }

    pub fn build(self, count: u64) -> BufferHandle {
        let size = count * std::mem::size_of::<T>() as u64;

        let buffer = Buffer::new::<T>(
            self.manager,
            self.label,
            size,
            self.usages,
            self.vertex_format,
        );

        self.manager.add_buffer(buffer)
    }

    pub fn build_init(self, init_data: Vec<T>) -> BufferHandle {
        let buffer = Buffer::new_init(
            self.manager,
            self.label,
            self.usages,
            init_data,
            self.vertex_format,
        );

        self.manager.add_buffer(buffer)
    }
}

impl<'a, T: Vertex> BufferBuilder<'a, T> {
    pub fn vertex(mut self) -> Self {
        self.usages |= BufferUsages::VERTEX;
        self.vertex_format = Some(vertex_format::<T>(VertexStepMode::Vertex));
        self
    }

    pub fn instance(mut self) -> Self {
        self.usages |= BufferUsages::VERTEX;
        self.vertex_format = Some(vertex_format::<T>(VertexStepMode::Instance));
        self
    }
}

impl<'a> BufferBuilder<'a, u16> {
    pub fn index(mut self) -> Self {
        self.usages |= BufferUsages::INDEX;
        self
    }
}

impl<'a> BufferBuilder<'a, u32> {
    pub fn index(mut self) -> Self {
        self.usages |= BufferUsages::INDEX;
        self
    }
}
