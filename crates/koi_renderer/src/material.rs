use crate::*;

pub struct Material {
    pub base_color: Color,
    pub base_color_texture: Option<koi_assets::Handle<crate::Texture>>,
    pub shader: koi_assets::Handle<Shader>,
}
