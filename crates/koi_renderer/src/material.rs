use std::collections::HashMap;

use crate::*;

// A useful chart for making materials:
// https://google.github.io/filament/images/material_chart.jpg

pub struct Material {
    pub shader: koi_assets::Handle<Shader>,
    pub base_color: Color,
    pub base_color_texture: Option<koi_assets::Handle<crate::Texture>>,
    pub metallicness: f32,
    pub perceptual_roughness: f32,
    pub metallic_roughness_texture: Option<koi_assets::Handle<crate::Texture>>,
    pub ambient_scale: f32,
    pub emissiveness: f32,
    pub reflectance: f32,
    pub normal_texture: Option<koi_assets::Handle<crate::Texture>>,
    //
    pub cube_map: Option<koi_assets::Handle<crate::CubeMap>>,
    //
    // TODO: This isn't the sort of thing that should be on a [Material], but for now it goes here.
    pub morph_weights: Vec<f32>,
    pub morphable_mesh_data: Option<koi_assets::Handle<MorphableMeshData>>,
    pub other_properties: HashMap<String, PropertyValue>,
}

#[derive(Clone)]
pub enum PropertyValue {
    F32(f32),
    Vec2(kmath::Vec2),
    Vec3(kmath::Vec3),
    Vec4(kmath::Vec4),
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
            normal_texture: None,
            //
            cube_map: None,
            //
            morph_weights: Vec::new(),
            morphable_mesh_data: None,
            other_properties: HashMap::new(),
        }
    }
}
