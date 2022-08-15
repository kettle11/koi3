use crate::Shader;
use koi_assets::Handle;

impl Shader {
    pub const UNLIT: Handle<Self> = Handle::from_index(0);
}

pub fn initialize_shaders(renderer: &mut crate::Renderer) -> koi_assets::AssetStore<Shader> {
    // Shader snippets
    renderer.register_shader_snippet(
        "standard_vertex",
        include_str!("shaders_glsl/snippets/standard_vertex.glsl"),
    );

    // Static shaders
    let shader = renderer
        .new_shader(
            include_str!("shaders_glsl/unlit.glsl"),
            crate::ShaderSettings {
                faces_to_render: kgraphics::FacesToRender::FrontAndBack,
                blending: None,
                ..Default::default()
            },
        )
        .unwrap();
    let asset_store = koi_assets::AssetStore::new(shader);

    asset_store
}
