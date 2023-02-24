use std::num::NonZeroU8;

use wgpu::{
    AddressMode,
    CompareFunction,
    FilterMode,
    Label,
    Sampler,
    SamplerBorderColor,
    SamplerDescriptor,
};

use crate::{handle::Handle, manager::RenderManager};

pub type TextureSampleHandle = Handle<TextureSampler>;

pub struct TextureSampler {
    sampler: Sampler,
}

impl TextureSampler {
    pub(crate) fn inner(&self) -> &Sampler {
        &self.sampler
    }
}

pub struct TextureSamplerBuilder<'a> {
    manager: &'a mut RenderManager,
    name: Label<'a>,
    address_mode_u: AddressMode,
    address_mode_v: AddressMode,
    address_mode_w: AddressMode,
    mag_filter: FilterMode,
    min_filter: FilterMode,
    mipmap_filter: FilterMode,
    lod_min_clamp: f32,
    lod_max_clamp: f32,
    compare: Option<CompareFunction>,
    anisotropy_clamp: Option<NonZeroU8>,
    border_color: Option<SamplerBorderColor>,
}

impl<'a> TextureSamplerBuilder<'a> {
    pub fn new(manager: &'a mut RenderManager, name: Label<'a>) -> TextureSamplerBuilder<'a> {
        let SamplerDescriptor {
            address_mode_u,
            address_mode_v,
            address_mode_w,
            mag_filter,
            min_filter,
            mipmap_filter,
            lod_min_clamp,
            lod_max_clamp,
            compare,
            anisotropy_clamp,
            border_color,
            ..
        } = SamplerDescriptor::default();
        TextureSamplerBuilder {
            manager,
            name,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            mag_filter,
            min_filter,
            mipmap_filter,
            lod_min_clamp,
            lod_max_clamp,
            compare,
            anisotropy_clamp,
            border_color,
        }
    }

    pub fn address_mode_u(mut self, address_mode: AddressMode) -> Self {
        self.address_mode_u = address_mode;
        self
    }

    pub fn address_mode_v(mut self, address_mode: AddressMode) -> Self {
        self.address_mode_v = address_mode;
        self
    }

    pub fn address_mode_w(mut self, address_mode: AddressMode) -> Self {
        self.address_mode_w = address_mode;
        self
    }

    pub fn mag_filter(mut self, filter_mode: FilterMode) -> Self {
        self.mag_filter = filter_mode;
        self
    }

    pub fn min_filter(mut self, filter_mode: FilterMode) -> Self {
        self.min_filter = filter_mode;
        self
    }

    pub fn mipmap_filter(mut self, filter_mode: FilterMode) -> Self {
        self.mipmap_filter = filter_mode;
        self
    }

    pub fn lod_min_clamp(mut self, min_lod: f32) -> Self {
        self.lod_min_clamp = min_lod;
        self
    }

    pub fn lod_max_clamp(mut self, max_lod: f32) -> Self {
        self.lod_max_clamp = max_lod;
        self
    }

    pub fn compare(mut self, func: CompareFunction) -> Self {
        self.compare = Some(func);
        self
    }

    pub fn anisotropy_clamp(mut self, val: u8) -> Self {
        self.anisotropy_clamp = NonZeroU8::new(val);
        self
    }

    pub fn border_color(mut self, color: SamplerBorderColor) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn build(self) -> TextureSampleHandle {
        self.manager.add_sampler(TextureSampler {
            sampler: self.manager.device.create_sampler(&SamplerDescriptor {
                label: self.name,
                address_mode_u: self.address_mode_u,
                address_mode_v: self.address_mode_v,
                address_mode_w: self.address_mode_w,
                mag_filter: self.mag_filter,
                min_filter: self.min_filter,
                mipmap_filter: self.mipmap_filter,
                lod_min_clamp: self.lod_min_clamp,
                lod_max_clamp: self.lod_max_clamp,
                compare: self.compare,
                anisotropy_clamp: self.anisotropy_clamp,
                border_color: self.border_color,
            }),
        })
    }
}
