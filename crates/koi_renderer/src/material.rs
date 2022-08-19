use crate::*;

pub struct Material {
    pub base_color: Color,
    pub base_color_texture: Option<koi_assets::Handle<crate::Texture>>,
    pub shader: koi_assets::Handle<Shader>,
    pub metallicness: f32,
    pub perceptual_roughness: f32,
    pub metallic_roughness_texture: Option<koi_assets::Handle<crate::Texture>>,
    pub ambient_scale: f32,
    pub emissiveness: f32,
    pub reflectance: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: Color::WHITE,
            base_color_texture: None,
            shader: Shader::PHYSICALLY_BASED,
            metallicness: 0.0,
            perceptual_roughness: 0.7,
            metallic_roughness_texture: None,
            ambient_scale: 1.0,
            emissiveness: 0.0,
            reflectance: 0.5,
        }
    }
}
