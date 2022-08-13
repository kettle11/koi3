use kgraphics::{GraphicsContextTrait, PipelineBuilderTrait, PipelineTrait};

pub struct Shader {
    pub pipeline: kgraphics::Pipeline,
    pub(crate) shader_render_properties: ShaderRenderProperties,
}

/*
pub(crate) struct SceneInfoUniformBlock {
    pub world_to_camera: kmath::Mat4,
    pub camera_to_screen: kmath::Mat4,
}
*/

/// Standard properties that a shader will use.
pub(crate) struct ShaderRenderProperties {
    // Uniform blocks
    // pub(crate) scene_info_uniform_block: kgraphics::UniformBlock<SceneInfoUniformBlock>,
    // Per-instance attributes
    pub(crate) local_to_world_instance_attribute: kgraphics::VertexAttribute<kmath::Mat4>,
    // Atributes
    pub(crate) position_attribute: kgraphics::VertexAttribute<kmath::Vec3>,
    pub(crate) normal_attribute: kgraphics::VertexAttribute<kmath::Vec3>,
    pub(crate) texture_coordinate_attribute: kgraphics::VertexAttribute<kmath::Vec2>,
    pub(crate) color_attribute: kgraphics::VertexAttribute<kmath::Vec4>,
    // Uniforms
    pub(crate) world_to_camera: kgraphics::Mat4Property,
    pub(crate) camera_to_screen: kgraphics::Mat4Property,
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
    pub fn new_shader(
        &mut self,
        source: &str,
        shader_settings: ShaderSettings,
    ) -> Result<Shader, ShaderError> {
        let (vertex_source, fragment_source) =
            crate::shader_parser::parse_shader(&std::collections::HashMap::new(), source, "");

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
            // scene_info_uniform_block: pipeline.get_uniform_block("ub_SceneInfo").unwrap(),
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
            // Uniforms
            world_to_camera: pipeline.get_mat4_property("world_to_camera").unwrap(),
            camera_to_screen: pipeline.get_mat4_property("camera_to_screen").unwrap(),
        };
        Ok(Shader {
            pipeline,
            shader_render_properties,
        })
    }
}
