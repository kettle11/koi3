use crate::Material;
use koi_assets::Handle;

impl Material {
    pub const UNLIT: Handle<Self> = Handle::from_index(0);
}

pub fn initialize_constant_materials(materials: &mut koi_assets::AssetStore<Material>) {
    materials.add_and_leak(
        Material {
            base_color_texture: None,
            shader: crate::Shader::UNLIT,
            base_color: kcolor::Color::WHITE,
        },
        &Material::UNLIT,
    );
}
