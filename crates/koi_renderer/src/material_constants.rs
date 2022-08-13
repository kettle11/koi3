use crate::Material;
use koi_assets::Handle;

impl Material {
    pub const TEST: Handle<Self> = Handle::from_index(0);
}

pub fn initialize_constant_materials(materials: &mut koi_assets::AssetStore<Material>) {
    materials.add_and_leak(
        Material {
            shader: crate::Shader::TEST,
        },
        &Material::TEST,
    );
}
