use std::ops::{Deref, DerefMut};

use bytemuck::{Pod, Zeroable};
use wgpu::VertexBufferLayout;
pub use wgpu::{VertexAttribute, VertexFormat, VertexStepMode};

pub(crate) const fn vertex_format<T: Vertex>() -> VertexBufferLayout<'static> {
    VertexBufferLayout {
        array_stride: std::mem::size_of::<T>() as u64,
        step_mode: T::STEP_MODE,
        attributes: T::FIELDS,
    }
}

/// Something that can be used in a vertex buffer
pub trait Vertex: Pod + Zeroable + Clone + Sized {
    /// A description of the different fields in the struct
    ///
    /// Has to be const because we need an `'static` reference<br>
    /// The place we use this needs a reference to the array and theres no easy way to do that without it being `'static`
    const FIELDS: &'static [VertexAttribute];
    /// How often the data sent to the shader should change
    const STEP_MODE: VertexStepMode;
}

pub trait VertexField {
    const FORMAT: VertexFormat;
}

macro_rules! vertex_fields {
    ($($kind: ty, $variant: ident),*) => {
        $(
            impl VertexField for $kind {
                const FORMAT: VertexFormat = VertexFormat::$variant;
            }
        )*
    };
}

#[repr(C)]
pub struct Norm<T>(T);

impl<T> Deref for Norm<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Norm<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

vertex_fields! {
    [u8; 2], Uint8x2,
    [u8; 4], Uint8x4,
    [i8; 2], Sint8x2,
    [i8; 4], Sint8x4,
    Norm<[u8; 2]>, Unorm8x2,
    Norm<[u8; 4]>, Unorm8x4,
    Norm<[i8; 2]>, Snorm8x2,
    Norm<[i8; 4]>, Snorm8x4,
    [u16; 2], Uint16x2,
    [u16; 4], Uint16x4,
    [i16; 2], Sint16x2,
    [i16; 4], Sint16x4,
    Norm<[u16; 2]>, Unorm16x2,
    Norm<[u16; 4]>, Unorm16x4,
    Norm<[i16; 2]>, Snorm16x2,
    Norm<[i16; 4]>, Snorm16x4,
    f32, Float32,
    [f32; 2], Float32x2,
    [f32; 3], Float32x3,
    [f32; 4], Float32x4,
    u32, Uint32,
    [u32; 2], Uint32x2,
    [u32; 3], Uint32x3,
    [u32; 4], Uint32x4,
    i32, Sint32,
    [i32; 2], Sint32x2,
    [i32; 3], Sint32x3,
    [i32; 4], Sint32x4,
    f64, Float64,
    [f64; 2], Float64x2,
    [f64; 3], Float64x3,
    [f64; 4], Float64x4
}
