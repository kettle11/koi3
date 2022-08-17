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
        shader: crate::Shader::UNLIT,
        base_color: kcolor::Color::WHITE,
        ..Default::default()
    };

    let mut materials = AssetStore::new(material);

    let material = Material {
        shader: crate::Shader::PHYSICALLY_BASED,
        base_color: kcolor::Color::WHITE,
        ..Default::default()
    };
    materials.add_and_leak(material, &Material::PHYSICALLY_BASED);

    materials
}
