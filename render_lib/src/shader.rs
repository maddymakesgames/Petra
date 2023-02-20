use wgpu::ShaderModule;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShaderHandle(pub(crate) usize);

pub struct Shader(pub(crate) ShaderModule);
