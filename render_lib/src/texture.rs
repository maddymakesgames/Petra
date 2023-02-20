#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextureHandle {
    id: usize,
}

impl TextureHandle {
    pub const FRAMEBUFFER: TextureHandle = TextureHandle { id: 0 };
}
