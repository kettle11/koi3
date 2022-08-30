use crate::*;
use koi_assets::*;
use koi_graphics_context::BufferUsage;
use koi_transform::Transform;

pub struct Renderer {
    pub raw_graphics_context: koi_graphics_context::GraphicsContext,
    render_pass_pool: Vec<RenderPass>,
    pub automatically_redraw: bool,
    pub(crate) shader_snippets: std::collections::HashMap<&'static str, &'static str>,
    color_space: kcolor::ColorSpace,
}

impl Renderer {
    pub(crate) fn new(
        raw_graphics_context: koi_graphics_context::GraphicsContext,
        color_space: koi_graphics_context::ColorSpace,
    ) -> Self {
        Self {
            raw_graphics_context,
            render_pass_pool: Vec::new(),
            automatically_redraw: true,
            shader_snippets: std::collections::HashMap::new(),
            color_space: match color_space {
                koi_graphics_context::ColorSpace::SRGB => kcolor::color_spaces::ENCODED_SRGB,
                koi_graphics_context::ColorSpace::DisplayP3 => {
                    kcolor::color_spaces::ENCODED_DISPLAY_P3
                }
            },
        }
    }
    pub fn begin_render_pass(
        &mut self,
        camera: &Camera,
        camera_transform: &Transform,
        view_width: f32,
        view_height: f32,
    ) -> RenderPass {
        if let Some(mut render_pass) = self.render_pass_pool.pop() {
            render_pass.camera = camera.clone();
            render_pass.camera_transform = *camera_transform;
            render_pass.meshes_to_draw.clear();
            render_pass.local_to_world_matrices.clear();
            render_pass.view_width = view_width;
            render_pass.view_height = view_height;
            render_pass.camera = camera.clone();
            render_pass.camera_transform = *camera_transform;
            render_pass.lights_bound = 0;
            render_pass.exposure_scale_factor =
                1.0 / camera.exposure.max_luminance_without_clipping();
            render_pass
        } else {
            RenderPass {
                meshes_to_draw: Vec::new(),
                local_to_world_matrices: Vec::new(),
                camera: camera.clone(),
                camera_transform: *camera_transform,
                view_width,
                view_height,
                color_space: self.color_space.clone(),
                light_info: [LightInfo::default(); MAX_BOUND_LIGHTS],
                lights_bound: 0,
                exposure_scale_factor: 1.0 / camera.exposure.max_luminance_without_clipping(),
            }
        }
    }
    pub fn submit_render_pass(
        &mut self,
        mut render_pass: RenderPass,
        meshes: &AssetStore<Mesh>,
        materials: &AssetStore<Material>,
        shaders: &AssetStore<Shader>,
        textures: &AssetStore<Texture>,
        cube_maps: &AssetStore<CubeMap>,
        morphable_mesh_data: &AssetStore<MorphableMeshData>,
    ) {
        render_pass.execute(
            &mut self.raw_graphics_context,
            meshes,
            materials,
            shaders,
            textures,
            cube_maps,
            morphable_mesh_data,
        );
        self.render_pass_pool.push(render_pass);
    }
}
pub struct RenderPass {
    camera: Camera,
    camera_transform: Transform,
    meshes_to_draw: Vec<(Handle<Material>, Handle<Mesh>, kmath::Mat4)>,
    local_to_world_matrices: Vec<kmath::Mat4>,
    view_width: f32,
    view_height: f32,
    color_space: kcolor::ColorSpace,
    light_info: [LightInfo; MAX_BOUND_LIGHTS],
    lights_bound: usize,
    exposure_scale_factor: f32,
}

impl RenderPass {
    pub fn add_directional_light(
        &mut self,
        transform: &Transform,
        directional_light: &crate::DirectionalLight,
    ) {
        if self.lights_bound < MAX_BOUND_LIGHTS {
            let light_info = &mut self.light_info[self.lights_bound];
            light_info.mode = 0; // 0 denotes directional
            light_info.position = transform.position;
            light_info.inverse_direction = -transform.forward();
            light_info.world_to_light = transform.local_to_world().inversed();

            // Preexpose the light based on the camera's exposure settings
            let light_color: kmath::Vec4 = directional_light.color.to_rgb_color(self.color_space);
            light_info.color_and_intensity = light_color.xyz()
                * directional_light.intensity_illuminance
                * self.exposure_scale_factor;

            self.lights_bound += 1;
        }
    }
    pub fn add_point_light(&mut self, transform: &Transform, point_light: &crate::PointLight) {
        if self.lights_bound < MAX_BOUND_LIGHTS {
            let light_info = &mut self.light_info[self.lights_bound];
            light_info.mode = 1; // // 1 denotes point
            light_info.position = transform.position;
            light_info.inverse_direction = -transform.forward();
            light_info.world_to_light = transform.local_to_world().inversed();

            // Preexpose the light based on the camera's exposure settings
            let light_color: kmath::Vec4 = point_light.color.to_rgb_color(self.color_space);
            light_info.color_and_intensity =
                light_color.xyz() * point_light.intensity_lumens * self.exposure_scale_factor;

            self.lights_bound += 1;
        }
    }
    pub fn draw_mesh(
        &mut self,
        mesh: &Handle<Mesh>,
        material: &Handle<Material>,
        transform: &Transform,
    ) {
        // Todo: Immediately cull mesh if outside frustum bounds.
        self.meshes_to_draw
            .push((material.clone(), mesh.clone(), transform.local_to_world()))
    }

    fn execute(
        &mut self,
        graphics: &mut koi_graphics_context::GraphicsContext,
        meshes: &AssetStore<Mesh>,
        materials: &AssetStore<Material>,
        shaders: &AssetStore<Shader>,
        textures: &AssetStore<Texture>,
        cube_maps: &AssetStore<CubeMap>,
        morphable_mesh_data: &AssetStore<MorphableMeshData>,
    ) {
        let mut command_buffer = graphics.new_command_buffer();

        /*
        let mut render_pass = command_buffer.begin_render_pass_with_framebuffer(
            &koi_graphics_context::Framebuffer::default(),
            self.camera
                .clear_color
                .map(|v| v.to_rgb_color(self.color_space).into()),
        );
        */
        let mut render_pass = command_buffer.begin_render_pass(
            self.camera
                .clear_color
                .map(|v| v.to_rgb_color(self.color_space).into()),
        );
        render_pass.set_viewport(0.0, 0.0, self.view_width as f32, self.view_height as f32);

        {
            let mut render_pass_executor = RenderPassExecutor {
                graphics,
                meshes,
                materials,
                shaders,
                textures,
                cube_maps,
                morphable_mesh_data,
                render_pass,
                current_material_and_shader: None,
                current_gpu_mesh: None,
                camera_position: self.camera_transform.position,
                world_to_camera: self.camera_transform.local_to_world().inversed(),
                camera_to_screen: self
                    .camera
                    .projection_matrix(self.view_width, self.view_height),
                this_render_pass: self,
            };

            render_pass_executor.execute();
        }
        graphics.execute_command_buffer(command_buffer);
    }
}

struct RenderPassExecutor<'a> {
    graphics: &'a mut koi_graphics_context::GraphicsContext,
    meshes: &'a AssetStore<Mesh>,
    materials: &'a AssetStore<Material>,
    shaders: &'a AssetStore<Shader>,
    textures: &'a AssetStore<Texture>,
    cube_maps: &'a AssetStore<CubeMap>,
    morphable_mesh_data: &'a AssetStore<MorphableMeshData>,
    render_pass: koi_graphics_context::RenderPass<'a>,
    current_material_and_shader: Option<(&'a Material, &'a Shader)>,
    current_gpu_mesh: Option<&'a GPUMesh>,
    camera_position: kmath::Vec3,
    world_to_camera: kmath::Mat4,
    camera_to_screen: kmath::Mat4,
    this_render_pass: &'a mut RenderPass,
}

impl<'a> RenderPassExecutor<'a> {
    fn render_group(&mut self) {
        if self.this_render_pass.local_to_world_matrices.is_empty() {
            return;
        }

        if let Some(gpu_mesh) = self.current_gpu_mesh {
            if let Some((material, shader)) = self.current_material_and_shader {
                self.render_pass.set_pipeline(&shader.pipeline);
                let shader_properties = &shader.shader_render_properties;

                // Bind the material properties
                {
                    let mut p_textures_enabled: i32 = 0;
                    if material.base_color_texture.is_some() {
                        p_textures_enabled |= 1 << 0;
                    }
                    if material.metallic_roughness_texture.is_some() {
                        p_textures_enabled |= 1 << 1;
                    }

                    let sp = &shader.shader_render_properties;

                    self.render_pass
                        .set_uniform(&sp.p_textures_enabled, p_textures_enabled);

                    self.render_pass.set_uniform(
                        &sp.p_base_color,
                        material
                            .base_color
                            .to_rgb_color(self.this_render_pass.color_space)
                            .into(),
                    );
                    let mut texture_unit = 0;

                    /*
                    self.render_pass.set_texture_property(
                        &sp.p_base_color_texture,
                        Some(material.base_color_texture.as_ref().map_or_else(
                            || &self.textures.get(&Texture::WHITE).0,
                            |t| &self.textures.get(t).0,
                        )),
                        texture_unit,
                    );
                    */
                    texture_unit += 1;

                    /*
                    self.render_pass.set_texture_property(
                        &sp.p_metallic_roughness_texture,
                        Some(material.metallic_roughness_texture.as_ref().map_or_else(
                            || &self.textures.get(&Texture::WHITE).0,
                            |t| &self.textures.get(t).0,
                        )),
                        texture_unit,
                    );
                    */
                    texture_unit += 1;

                    self.render_pass
                        .set_uniform(&sp.p_metallic, material.metallicness);

                    // Roughness is multiplied by itself to make roughness increase in a more *perceptually* linear way.
                    // TODO: Investigate how this matches with other software.
                    self.render_pass.set_uniform(
                        &sp.p_roughness,
                        material.perceptual_roughness * material.perceptual_roughness,
                    );
                    self.render_pass
                        .set_uniform(&sp.p_ambient, material.ambient_scale);
                    self.render_pass
                        .set_uniform(&sp.p_emissive, material.emissiveness);
                    self.render_pass
                        .set_uniform(&sp.p_reflectance, material.reflectance);

                    /*
                    self.render_pass.set_cube_map_property(
                        &sp.p_cube_map,
                        Some(material.cube_map.as_ref().map_or_else(
                            || &self.cube_maps.get(&Handle::PLACEHOLDER).texture,
                            |t| &self.cube_maps.get(t).texture,
                        )),
                        texture_unit,
                    );
                    */
                    texture_unit += 1;

                    /*
                    if let Some(morphable_mesh_data) = material.morphable_mesh_data.as_ref() {
                        let morphable_mesh_data = self.morphable_mesh_data.get(morphable_mesh_data);
                        let texture = self
                            .textures
                            .get(&morphable_mesh_data.morph_targets_texture);

                        self.render_pass.set_texture_property(
                            &shader
                                .pipeline
                                .get_texture_property("u_morph_targets")
                                .unwrap(),
                            Some(&texture.0),
                            texture_unit,
                        );

                        // TODO: Replace this with an array of values.
                        self.render_pass.set_vec3_property(
                            &shader
                                .pipeline
                                .get_vec3_property("u_morph_target_influences")
                                .unwrap(),
                            (
                                material.morph_weights.get(0).copied().unwrap_or(0.0),
                                material.morph_weights.get(0).copied().unwrap_or(0.0),
                                material.morph_weights.get(0).copied().unwrap_or(0.0),
                            ),
                        )
                    }
                    */
                }

                // Bind the mesh for this group.
                {
                    self.render_pass.set_vertex_attribute(
                        &shader_properties.position_attribute,
                        Some(&gpu_mesh.positions),
                    );
                    self.render_pass.set_vertex_attribute(
                        &shader_properties.normal_attribute,
                        gpu_mesh.normals.as_ref(),
                    );
                    self.render_pass.set_vertex_attribute(
                        &shader_properties.texture_coordinate_attribute,
                        gpu_mesh.texture_coordinates.as_ref(),
                    );

                    // If no color is provided set all vertex colors to white.
                    if let Some(colors) = gpu_mesh.colors.as_ref() {
                        self.render_pass
                            .set_vertex_attribute(&shader_properties.color_attribute, Some(colors));
                    } else {
                        /*
                        self.render_pass.set_vertex_attribute_to_constant(
                            &shader_properties.color_attribute,
                            &[1.0, 1.0, 1.0, 1.0],
                        );
                        */
                    }
                }

                // Upload a new buffer for the thing being rendered.
                // TODO: Investigate if this is inefficient for single objects.
                let local_to_world_data = self.graphics.new_buffer(
                    &self.this_render_pass.local_to_world_matrices,
                    BufferUsage::Data,
                );

                // self.render_pass.set_instance_attribute(
                //     &shader_properties.local_to_world_instance_attribute,
                //     Some(&local_to_world_data),
                // );

                self.render_pass.draw(
                    Some(&gpu_mesh.index_buffer),
                    gpu_mesh.index_start..gpu_mesh.index_end,
                    self.this_render_pass.local_to_world_matrices.len() as u32,
                );
                self.this_render_pass.local_to_world_matrices.clear();
            }
        }
    }

    fn execute(&mut self) {
        // Bind global data.
        {
            let data_buffer = self.graphics.new_buffer(
                &[SceneInfoUniformBlock {
                    p_world_to_camera: self.world_to_camera,
                    p_camera_to_screen: self.camera_to_screen,
                    p_camera_position: self.camera_position,
                    p_dither_scale: 1.0,
                    p_fog_start: 0.0,
                    p_fog_end: 100.0,
                    p_exposure: self.this_render_pass.exposure_scale_factor,
                    light_count: self.this_render_pass.lights_bound as _,
                    spherical_harmonic_weights: self
                        .cube_maps
                        .get(&Handle::from_index(0))
                        .spherical_harmonics
                        .convolve_with_cos_irradiance_and_premultiply_constants(
                            self.this_render_pass.exposure_scale_factor,
                        ),
                    // TODO: Don't do a clone here
                    lights: self.this_render_pass.light_info.clone(),
                }],
                BufferUsage::Data,
            );
            self.render_pass.set_uniform_block(0, Some(&data_buffer));
        }

        // Sort meshes by material, then mesh.
        // TODO: This could be made more efficient by sorting by pipeline as well.
        // As-is small material variants will incur a cost.
        self.this_render_pass
            .meshes_to_draw
            .sort_by_key(|v| (v.0.clone(), v.1.clone()));

        let mut current_mesh = None;
        let mut current_material = None;

        let mut meshes_to_draw = Vec::new();
        std::mem::swap(
            &mut meshes_to_draw,
            &mut self.this_render_pass.meshes_to_draw,
        );

        // Renders batches of meshes that share the same material.
        for (material_handle, mesh_handle, local_to_world_matrix) in meshes_to_draw.iter() {
            let mut change_material = false;
            let mut change_mesh = None;

            if Some(material_handle) != current_material {
                // Changing materials, draw the current mesh group.
                self.render_group();
                change_material = true;
            }

            if Some(mesh_handle) != current_mesh {
                if let Some(gpu_mesh) = &self.meshes.get(mesh_handle).gpu_mesh {
                    // Changing meshes, draw the current mesh group.
                    self.render_group();
                    change_mesh = Some(gpu_mesh);
                }
            }

            if change_material {
                let material = self.materials.get(material_handle);
                let shader = self.shaders.get(&material.shader);
                self.current_material_and_shader = Some((material, shader));
                current_material = Some(material_handle);
            }

            if let Some(gpu_mesh) = change_mesh {
                self.current_gpu_mesh = Some(gpu_mesh);
                current_mesh = Some(mesh_handle);
            }

            self.this_render_pass
                .local_to_world_matrices
                .push(*local_to_world_matrix);
        }

        self.render_group();
        std::mem::swap(
            &mut meshes_to_draw,
            &mut self.this_render_pass.meshes_to_draw,
        );
    }
}
