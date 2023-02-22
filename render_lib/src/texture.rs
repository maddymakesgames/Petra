use std::{any::TypeId, marker::PhantomData, num::NonZeroU32, sync::Arc};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    Device,
    Extent3d,
    ImageDataLayout,
    Label,
    Queue,
    SurfaceConfiguration,
    Texture as RawTexture,
    TextureDescriptor,
    TextureDimension,
    TextureFormat,
    TextureUsages,
};

use crate::{handle::Handle, manager::RenderManager};

pub type TextureHandle = Handle<Texture>;
pub const FRAMEBUFFER: TextureHandle = Handle::new(0);

pub struct Texture {
    name: Option<String>,
    texture: RawTexture,
    device: Arc<Device>,
    queue: Arc<Queue>,
    size: TextureSize,
    mip_level_count: u32,
    sample_count: u32,
    data_type: TypeId,
}

impl Texture {
    pub(crate) fn on_resize(&mut self, config: &SurfaceConfiguration) {
        if let TextureSize::Surface | TextureSize::ScaledSurface(..) = self.size {
            self.recreate(self.size.get_size(config))
        }
    }

    pub fn resize_1d(&mut self, width: u32) {
        self.resize(TextureSize::D1(width))
    }

    pub fn resize_2d(&mut self, width: u32, height: u32) {
        self.resize(TextureSize::D2(width, height))
    }

    pub fn resize_3d(&mut self, width: u32, height: u32, depth_or_array_len: u32) {
        self.resize(TextureSize::D3(width, height, depth_or_array_len))
    }

    pub fn write_data<T: TextureContents>(
        &mut self,
        data: &[T::Data],
        config: &SurfaceConfiguration,
    ) {
        if TypeId::of::<T>() != self.data_type {
            panic!(
                "Tried to write to texture with a type that did not match the one it was declared \
                 with"
            )
        }

        let byte_slice = bytemuck::cast_slice(data);
        self.queue.write_texture(
            self.texture.as_image_copy(),
            byte_slice,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: self
                    .size
                    .get_bytes_per_row(std::mem::size_of::<T::Data>() as u32, config),
                rows_per_image: self.size.get_rows_per_image(),
            },
            self.size.get_size(config),
        );
    }

    fn resize(&mut self, size: TextureSize) {
        if let TextureSize::Surface | TextureSize::ScaledSurface(..) = size {
            panic!("Texture size can only be set to be relative to the surface size at creation");
        } else {
            self.size = match (self.size, size) {
                (TextureSize::D1(_), TextureSize::D1(x)) => TextureSize::D1(x),
                (TextureSize::D2(..), TextureSize::D2(x, y)) => TextureSize::D2(x, y),
                (TextureSize::D3(..), TextureSize::D3(x, y, z)) => TextureSize::D3(x, y, z),
                _ => panic!("Tried to resize a texture to be a different dimension"),
            }
        }
    }

    fn recreate(&mut self, size: Extent3d) {
        let format = self.texture.format();
        let usage = self.texture.usage();
        let old_texture = std::mem::replace(
            &mut self.texture,
            self.device.create_texture(&TextureDescriptor {
                label: self.name.as_deref(),
                size,
                mip_level_count: self.mip_level_count,
                sample_count: self.sample_count,
                dimension: self.size.get_dimension(),
                format,
                usage,
                view_formats: &[],
            }),
        );

        old_texture.destroy();
    }
}
pub struct TextureBuilder<'a, T: TextureContents> {
    manager: &'a mut RenderManager,
    label: Label<'a>,
    size: Option<TextureSize>,
    mip_level_count: u32,
    sample_count: u32,
    usage: TextureUsages,
    __texture_format: PhantomData<T>,
}


impl<'a, T: TextureContents> TextureBuilder<'a, T> {
    pub(crate) fn new(manager: &'a mut RenderManager, label: Label<'a>) -> TextureBuilder<'a, T> {
        TextureBuilder {
            label,
            size: None,
            manager,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::empty(),
            __texture_format: PhantomData,
        }
    }

    pub fn size_1d(mut self, width: u32) -> Self {
        self.size = Some(TextureSize::D1(width));
        self
    }

    pub fn size_2d(mut self, width: u32, height: u32) -> Self {
        self.size = Some(TextureSize::D2(width, height));
        self
    }

    pub fn size_3d(mut self, width: u32, height: u32, depth_or_array_len: u32) -> Self {
        self.size = Some(TextureSize::D3(width, height, depth_or_array_len));
        self
    }

    pub fn size_framebuffer(mut self) -> Self {
        self.size = Some(TextureSize::Surface);
        self
    }

    pub fn size_scaled_framebuffer(mut self, width_scale: f32, height_scale: f32) -> Self {
        self.size = Some(TextureSize::ScaledSurface(width_scale, height_scale));
        self
    }

    pub fn build(self) -> TextureHandle {
        let size = self
            .size
            .expect("Trying to build texture with no specified size");

        let texture = self.manager.device.create_texture(&TextureDescriptor {
            label: self.label,
            size: size.get_size(&self.manager.config),
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            dimension: size.get_dimension(),
            format: T::FORMAT,
            usage: self.usage,
            // TODO: support extra view formats
            view_formats: &[],
        });

        self.manager.textures.add(Texture {
            name: self.label.map(|s| s.to_owned()),
            texture,
            device: self.manager.device.clone(),
            queue: self.manager.queue.clone(),
            size,
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            data_type: TypeId::of::<T>(),
        })
    }
}

#[derive(Clone, Copy)]
enum TextureSize {
    D1(u32),
    D2(u32, u32),
    D3(u32, u32, u32),
    Surface,
    ScaledSurface(f32, f32),
}

impl TextureSize {
    pub fn get_size(&self, config: &SurfaceConfiguration) -> Extent3d {
        match self {
            TextureSize::D1(x) => Extent3d {
                width: *x,
                height: 1,
                depth_or_array_layers: 1,
            },
            TextureSize::D2(x, y) => Extent3d {
                width: *x,
                height: *y,
                depth_or_array_layers: 1,
            },
            TextureSize::D3(x, y, z) => Extent3d {
                width: *x,
                height: *y,
                depth_or_array_layers: *z,
            },
            TextureSize::Surface => Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            TextureSize::ScaledSurface(x_scale, y_scale) => Extent3d {
                width: (config.width as f32 * x_scale) as u32,
                height: (config.height as f32 * y_scale) as u32,
                depth_or_array_layers: 1,
            },
        }
    }

    pub fn get_dimension(&self) -> TextureDimension {
        match &self {
            TextureSize::D1(_) => TextureDimension::D1,
            TextureSize::D2(..) | TextureSize::Surface | TextureSize::ScaledSurface(..) =>
                TextureDimension::D2,
            TextureSize::D3(..) => TextureDimension::D3,
        }
    }

    pub fn get_bytes_per_row(
        &self,
        bytes: u32,
        config: &SurfaceConfiguration,
    ) -> Option<NonZeroU32> {
        match &self {
            TextureSize::D1(_) => None,
            TextureSize::D2(x, _) => NonZeroU32::new(*x * bytes),
            TextureSize::D3(x, ..) => NonZeroU32::new(*x * bytes),
            TextureSize::Surface => NonZeroU32::new(bytes * config.width),
            TextureSize::ScaledSurface(x, _) =>
                NonZeroU32::new(bytes * (config.width as f32 * x) as u32),
        }
    }

    fn get_rows_per_image(&self) -> Option<NonZeroU32> {
        match &self {
            TextureSize::D1(_)
            | TextureSize::D2(..)
            | TextureSize::Surface
            | TextureSize::ScaledSurface(..) => None,
            TextureSize::D3(_, y, _) => NonZeroU32::new(*y),
        }
    }
}

pub trait TextureContents: 'static {
    const FORMAT: TextureFormat;
    type Data: 'static + Clone + Copy + Pod + Zeroable;
}

pub struct Norm<T>(T);
pub struct Stencil<T>(T);
pub struct Depth<T>(T);
pub struct Srgb<T>(T);

pub struct Bgra<T>(T);

macro_rules! formats {
    ($($kind: ty, $format: ident),*) => {
        $(
            impl TextureContents for $kind {
                const FORMAT: TextureFormat = TextureFormat::$format;
                type Data = Self;
            }
        )*
    };
    ($($kind: ty, $data: ty, $format: ident),*) => {
        $(
            impl TextureContents for $kind {
                const FORMAT: TextureFormat = TextureFormat::$format;
                type Data = $data;
            }
        )*
    };
}

formats! {
    u8, R8Uint,
    i8, R8Sint,
    u16, R16Uint,
    i16, R16Sint,
    [u8; 2], Rg8Uint,
    [i8; 2], Rg8Sint,
    u32, R32Uint,
    i32, R32Sint,
    f32, R32Float,
    [u16; 2], Rg16Uint,
    [i16; 2], Rg16Sint,
    [u8; 4], Rgba8Uint,
    [i8; 4], Rgba8Sint,
    [u32; 2], Rg32Uint,
    [i32; 2], Rg32Sint,
    [u16; 4], Rgba16Uint,
    [i16; 4], Rgba16Sint,
    [u32; 4], Rgba32Uint,
    [i32; 4], Rgba32Sint,
    [f32; 4], Rgba32Float
}

formats! {
    Norm<u8>, u8, R8Unorm,
    Norm<i8>, u8, R8Snorm,
    Norm<u16>, u16, R16Unorm,
    Norm<i16>, i16, R16Snorm,
    Norm<[u8; 2]>, [u8; 2], Rg8Unorm,
    Norm<[i8; 2]>, [i8; 2], Rg8Snorm,
    Norm<[u16; 2]>, [u16; 2], Rg16Unorm,
    Norm<[i16; 2]>, [i16; 2], Rg16Snorm,
    Norm<[u8; 4]>, [u8; 4], Rgba8Unorm,
    Srgb<Norm<[u8; 4]>>, [u8; 4], Rgba8UnormSrgb,
    Norm<[i8; 4]>, [i8; 4], Rgba8Snorm,
    Bgra<Norm<[u8; 4]>>, [u8; 4], Bgra8Unorm,
    Srgb<Bgra<Norm<[u8; 4]>>>, [u8; 4], Bgra8UnormSrgb,
    Norm<[u16; 4]>, [u16; 4], Rgba16Unorm,
    Norm<[i16; 4]>, [i16; 4], Rgba16Snorm,
    Stencil<u8>, u8, Stencil8,
    Stencil<i8>, i8, Stencil8,
    Depth<u16>, u16, Depth16Unorm,
    Depth<i16>, i16, Depth16Unorm,
    Depth<Norm<u16>>, u16, Depth16Unorm,
    Depth<Norm<i16>>, i16, Depth16Unorm,
    Depth<f32>, f32, Depth32Float
}
