use kgraphics::{GraphicsContextTrait, PipelineBuilderTrait, PipelineTrait};

pub struct Shader {
    pub pipeline: kgraphics::Pipeline,
    pub(crate) shader_render_properties: ShaderRenderProperties,
}

#[allow(unused)]
#[repr(C)]
pub(crate) struct SceneInfoUniformBlock {
    pub p_world_to_camera: kmath::Mat4,
    pub p_camera_to_screen: kmath::Mat4,
    pub p_camera_position: kmath::Vec3,
    pub p_dither_scale: f32,
    pub p_fog_start: f32,
    pub p_fog_end: f32,
}

/// Standard properties that a shader will use.
pub(crate) struct ShaderRenderProperties {
    // Uniform blocks
    // pub(crate) ub_scene_info: kgraphics::UniformBlock<SceneInfoUniformBlock>,
    // Per-instance attributes
    pub(crate) local_to_world_instance_attribute: kgraphics::VertexAttribute<kmath::Mat4>,
    // Atributes
    pub(crate) position_attribute: kgraphics::VertexAttribute<kmath::Vec3>,
    pub(crate) normal_attribute: kgraphics::VertexAttribute<kmath::Vec3>,
    pub(crate) texture_coordinate_attribute: kgraphics::VertexAttribute<kmath::Vec2>,
    pub(crate) color_attribute: kgraphics::VertexAttribute<kmath::Vec4>,
    // Per-object Uniforms
    pub(crate) p_base_color: kgraphics::Vec4Property,
    pub(crate) p_base_color_texture: kgraphics::TextureProperty,
    //
    pub(crate) p_metallic: kgraphics::FloatProperty,
    pub(crate) p_roughness: kgraphics::FloatProperty,
    pub(crate) p_ambient: kgraphics::FloatProperty,
    pub(crate) p_emissive: kgraphics::FloatProperty,
}

#[derive(Clone, Copy)]
pub struct ShaderSettings {
    pub faces_to_render: kgraphics::FacesToRender,
    pub blending: Option<(kgraphics::BlendFactor, kgraphics::BlendFactor)>,
    pub depth_test: kgraphics::DepthTest,
}

impl Default for ShaderSettings {
    fn default() -> Self {
        Self {
            faces_to_render: kgraphics::FacesToRender::Front,
            blending: None,
            depth_test: kgraphics::DepthTest::LessOrEqual,
        }
    }
}

#[derive(Debug)]
pub enum ShaderError {
    MissingVertexSection,
    MissingFragmentSection,
    VertexCompilationError(String),
    FragmentCompilationError(String),
    PipelineCompilationError(String),
}

impl crate::Renderer {
    pub fn register_shader_snippet(&mut self, name: &'static str, snippet: &'static str) {
        self.shader_snippets.insert(name, snippet);
    }
    pub fn new_shader(
        &mut self,
        source: &str,
        shader_settings: ShaderSettings,
    ) -> Result<Shader, ShaderError> {
        let (vertex_source, fragment_source) =
            crate::shader_parser::parse_shader(&self.shader_snippets, source, "");

        let vertex_function = self
            .raw_graphics_context
            .new_vertex_function(&vertex_source)
            .map_err(ShaderError::VertexCompilationError)?;
        let fragment_function = self
            .raw_graphics_context
            .new_fragment_function(&fragment_source)
            .map_err(ShaderError::FragmentCompilationError)?;

        let pipeline = self
            .raw_graphics_context
            .new_pipeline(
                vertex_function,
                fragment_function,
                /* Todo: This arbitrary pixel format is a problem */
                kgraphics::PixelFormat::RG8Unorm,
            )
            // For now all pipelines just have alpha blending by default.
            .blending(shader_settings.blending)
            .faces_to_render(shader_settings.faces_to_render)
            .depth_test(shader_settings.depth_test)
            .build()
            .map_err(ShaderError::PipelineCompilationError)?;

        let shader_render_properties = ShaderRenderProperties {
            // ub_scene_info: pipeline.get_uniform_block("ub_scene_info").unwrap(),
            // Per-instance attributes
            local_to_world_instance_attribute: pipeline
                .get_vertex_attribute("ia_local_to_world")
                .unwrap(),
            // Per-vertex attributes
            position_attribute: pipeline.get_vertex_attribute("a_position").unwrap(),
            normal_attribute: pipeline.get_vertex_attribute("a_normal").unwrap(),
            texture_coordinate_attribute: pipeline
                .get_vertex_attribute("a_texture_coordinate")
                .unwrap(),
            color_attribute: pipeline.get_vertex_attribute("a_color").unwrap(),
            // Per-object Uniforms
            p_base_color: pipeline.get_vec4_property("p_base_color").unwrap(),
            p_base_color_texture: pipeline
                .get_texture_property("p_base_color_texture")
                .unwrap(),
            p_metallic: pipeline.get_float_property("p_metallic").unwrap(),
            p_roughness: pipeline.get_float_property("p_roughness").unwrap(),
            p_ambient: pipeline.get_float_property("p_ambient").unwrap(),
            p_emissive: pipeline.get_float_property("p_emissive").unwrap(),
        };
        Ok(Shader {
            pipeline,
            shader_render_properties,
        })
    }
}

impl koi_assets::AssetTrait for Shader {
    type Settings = crate::ShaderSettings;
}
