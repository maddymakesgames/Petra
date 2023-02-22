use crate::handle::Handle;

pub type TextureHandle = Handle<Texture>;
pub const FRAMEBUFFER: TextureHandle = Handle::new(0);

pub struct Texture {}
