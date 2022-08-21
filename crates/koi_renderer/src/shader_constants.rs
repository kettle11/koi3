use crate::{Shader, ShaderSettings};
use koi_assets::Handle;
use koi_resources::Resources;

impl Shader {
    pub const UNLIT: Handle<Self> = Handle::from_index(0);
    pub const PHYSICALLY_BASED: Handle<Self> = Handle::from_index(1);
    pub(crate) const EQUIRECTANGULAR_TO_CUBE_MAP: Handle<Self> = Handle::from_index(2);
    pub const SKYBOX: Handle<Self> = Handle::from_index(3);
}

pub fn initialize_shaders(renderer: &mut crate::Renderer, resources: &mut Resources) {
    // Shader snippets
    renderer.register_shader_snippet(
        "scene_info",
        include_str!("shaders_glsl/snippets/scene_info.glsl"),
    );

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

    async fn load_shader(path: String, _settings: ShaderSettings) -> Option<String> {
        let bytes = koi_fetch::fetch_bytes(&path)
            .await
            .unwrap_or_else(|_| panic!("Failed to open file: {}", path));

        Some(std::str::from_utf8(&bytes).ok()?.to_owned())
    }
    fn finalize_shader_load(
        source: String,
        settings: ShaderSettings,
        resources: &Resources,
    ) -> Option<Shader> {
        let result = resources
            .get::<crate::Renderer>()
            .new_shader(&source, settings);
        match result {
            Ok(s) => Some(s),
            Err(e) => {
                println!("Shader compilation error: {:#?}", e);
                None
            }
        }
    }

    let mut asset_store =
        koi_assets::AssetStore::new_with_load_functions(shader, load_shader, finalize_shader_load);

    let shader = renderer
        .new_shader(
            include_str!("shaders_glsl/physically_based.glsl"),
            crate::ShaderSettings {
                faces_to_render: kgraphics::FacesToRender::FrontAndBack,
                blending: None,
                ..Default::default()
            },
        )
        .unwrap();

    asset_store.add_and_leak(shader, &Shader::PHYSICALLY_BASED);

    let shader = renderer
        .new_shader(
            include_str!("shaders_glsl/equirectangular_to_cubemap.glsl"),
            crate::ShaderSettings {
                // Todo: is this necessary?
                depth_test: kgraphics::DepthTest::LessOrEqual,
                blending: None,
                ..Default::default()
            },
        )
        .unwrap();

    asset_store.add_and_leak(shader, &Shader::EQUIRECTANGULAR_TO_CUBE_MAP);

    let shader = renderer
        .new_shader(
            include_str!("shaders_glsl/skybox.glsl"),
            crate::ShaderSettings {
                faces_to_render: kgraphics::FacesToRender::FrontAndBack,
                ..Default::default()
            },
        )
        .unwrap();
    asset_store.add_and_leak(shader, &Shader::SKYBOX);

    resources.add(asset_store);
}
