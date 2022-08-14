use crate::Shader;
use koi_assets::Handle;

impl Shader {
    pub const UNLIT: Handle<Self> = Handle::from_index(0);
}

pub fn initialize_constant_shader(
    renderer: &mut crate::Renderer,
    shaders: &mut koi_assets::AssetStore<Shader>,
) {
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
    shaders.add_and_leak(shader, &Shader::UNLIT);
}
