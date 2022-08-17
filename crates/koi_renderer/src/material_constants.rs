use crate::Material;
use koi_assets::{AssetStore, AssetTrait, Handle};

impl Material {
    pub const UNLIT: Handle<Self> = Handle::from_index(0);
    pub const PHYSICALLY_BASED: Handle<Self> = Handle::from_index(1);
}

impl AssetTrait for Material {
    type Settings = ();
}

pub fn initialize_materials() -> AssetStore<Material> {
    let material = Material {
        base_color_texture: None,
        shader: crate::Shader::UNLIT,
        base_color: kcolor::Color::WHITE,
    };

    let mut materials = AssetStore::new(material);

    let material = Material {
        base_color_texture: None,
        shader: crate::Shader::PHYSICALLY_BASED,
        base_color: kcolor::Color::WHITE,
    };
    materials.add_and_leak(material, &Material::PHYSICALLY_BASED);

    materials
}
