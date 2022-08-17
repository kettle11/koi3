use crate::Material;
use koi_assets::{AssetStore, AssetTrait, Handle};

impl Material {
    pub const UNLIT: Handle<Self> = Handle::from_index(0);
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

    let materials = AssetStore::new(material);
    materials
}
