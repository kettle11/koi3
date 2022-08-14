use crate::Shader;
use koi_assets::Handle;

impl Shader {
    pub const TEST: Handle<Self> = Handle::from_index(0);
}

pub fn initialize_constant_shader(
    renderer: &mut crate::Renderer,
    shaders: &mut koi_assets::AssetStore<Shader>,
) {
    /*
    renderer.register_shader_snippet(
        "fragment_out",
        include_str!("shaders_glsl/snippets/fragment_out.glsl"),
    );
    */
    let shader = renderer
        .new_shader(
            include_str!("shaders_glsl/test.glsl"),
            crate::ShaderSettings::default(),
        )
        .unwrap();
    shaders.add_and_leak(shader, &Shader::TEST);
}
