use koi_graphics_context::BufferDataTrait;

pub struct Shader {
    pub pipeline: koi_graphics_context::Pipeline,
    pub(crate) shader_render_properties: ShaderRenderProperties,
    pub shader_settings: ShaderSettings,
}

pub const MAX_BOUND_LIGHTS: usize = 100;

#[allow(unused)]
#[repr(C)]
pub(crate) struct SceneInfoUniformBlock {
    pub p_world_to_camera: kmath::Mat4,
    pub p_camera_to_screen: kmath::Mat4,
    pub p_camera_position: kmath::Vec3,
    pub p_dither_scale: f32,
    pub p_fog_start: f32,
    pub p_fog_end: f32,
    pub p_exposure: f32,
    pub light_count: u32,
    pub spherical_harmonic_weights: [kmath::Vec4; 9],
    pub lights: [LightInfo; MAX_BOUND_LIGHTS],
}

impl BufferDataTrait for SceneInfoUniformBlock {}

#[allow(unused)]
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct LightInfo {
    pub position: kmath::Vec3,
    pub radius: f32,
    pub inverse_direction: kmath::Vec3,
    pub ambient: f32,
    pub color_and_intensity: kmath::Vec3,
    pub mode: i32,
    pub world_to_light: kmath::Mat4,
}

impl Default for LightInfo {
    fn default() -> Self {
        Self {
            position: kmath::Vec3::ZERO,
            radius: 0.0,
            inverse_direction: kmath::Vec3::ZERO,
            ambient: 0.0,
            color_and_intensity: kmath::Vec3::ZERO,
            mode: 0,
            world_to_light: kmath::Mat4::ZERO,
        }
    }
}

/// Standard properties that a shader will use.
pub(crate) struct ShaderRenderProperties {
    // Uniform blocks
    // pub(crate) ub_scene_info: koi_graphics_context::UniformBlock<SceneInfoUniformBlock>,
    // Per-instance attributes
    #[allow(unused)]
    pub(crate) local_to_world_instance_attribute:
        koi_graphics_context::VertexAttribute<kmath::Mat4>,
    // Atributes
    pub(crate) position_attribute: koi_graphics_context::VertexAttribute<kmath::Vec3>,
    pub(crate) normal_attribute: koi_graphics_context::VertexAttribute<kmath::Vec3>,
    pub(crate) texture_coordinate_attribute: koi_graphics_context::VertexAttribute<kmath::Vec2>,
    pub(crate) color_attribute: koi_graphics_context::VertexAttribute<kmath::Vec4>,
    // Per-object Uniforms
    pub(crate) p_base_color: koi_graphics_context::Uniform<kmath::Vec4>,
    // pub(crate) p_base_color_texture: koi_graphics_context::TextureProperty,
    //
    pub(crate) p_metallic: koi_graphics_context::Uniform<f32>,
    pub(crate) p_roughness: koi_graphics_context::Uniform<f32>,
    // pub(crate) p_metallic_roughness_texture: koi_graphics_context::TextureProperty,
    //
    pub(crate) p_ambient: koi_graphics_context::Uniform<f32>,
    pub(crate) p_emissive: koi_graphics_context::Uniform<f32>,
    pub(crate) p_reflectance: koi_graphics_context::Uniform<f32>,
    //
    pub(crate) p_textures_enabled: koi_graphics_context::Uniform<i32>,
    // Optional extras:
    // pub(crate) p_cube_map: koi_graphics_context::CubeMapProperty,
}

#[derive(Clone, Copy)]
pub struct ShaderSettings {
    pub faces_to_render: koi_graphics_context::FacesToRender,
    pub blending: Option<(
        koi_graphics_context::BlendFactor,
        koi_graphics_context::BlendFactor,
    )>,
    pub depth_test: koi_graphics_context::DepthTest,
}

impl Default for ShaderSettings {
    fn default() -> Self {
        Self {
            faces_to_render: koi_graphics_context::FacesToRender::Front,
            blending: None,
            depth_test: koi_graphics_context::DepthTest::LessOrEqual,
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

        let pipeline = self
            .raw_graphics_context
            .new_pipeline(
                &vertex_source,
                &fragment_source,
                /* Todo: This arbitrary pixel format is a problem */
                koi_graphics_context::PipelineSettings {
                    blending: shader_settings.blending,
                    faces_to_render: shader_settings.faces_to_render,
                    depth_test: shader_settings.depth_test,
                },
            )
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
            p_base_color: pipeline.get_uniform("p_base_color").unwrap(),
            // p_base_color_texture: pipeline
            //     .get_texture_property("p_base_color_texture")
            //     .unwrap(),
            //
            p_metallic: pipeline.get_uniform("p_metallic").unwrap(),
            p_roughness: pipeline.get_uniform("p_roughness").unwrap(),
            // p_metallic_roughness_texture: pipeline
            //     .get_texture_property("p_metallic_roughness_texture")
            //     .unwrap(),
            //
            p_ambient: pipeline.get_uniform("p_ambient").unwrap(),
            p_emissive: pipeline.get_uniform("p_emissive").unwrap(),
            p_reflectance: pipeline.get_uniform("p_reflectance").unwrap(),
            //
            p_textures_enabled: pipeline.get_uniform("p_textures_enabled").unwrap(),
            //
            //  p_cube_map: pipeline.get_cube_map_property("p_cube_map").unwrap(),
        };
        Ok(Shader {
            pipeline,
            shader_render_properties,
            shader_settings,
        })
    }
}

impl koi_assets::AssetTrait for Shader {
    type Settings = crate::ShaderSettings;
}
