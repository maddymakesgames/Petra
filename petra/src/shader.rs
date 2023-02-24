use wgpu::ShaderModule;

use crate::handle::Handle;

pub type ShaderHandle = Handle<Shader>;

pub struct Shader(pub(crate) ShaderModule);
