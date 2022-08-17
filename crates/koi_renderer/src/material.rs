use crate::*;

pub struct Material {
    pub base_color: Color,
    pub base_color_texture: Option<koi_assets::Handle<crate::Texture>>,
    pub shader: koi_assets::Handle<Shader>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: Color::WHITE,
            base_color_texture: None,
            shader: Shader::PHYSICALLY_BASED,
        }
    }
}
