use std::char::MAX;

use crate::*;
use kgraphics::{CommandBufferTrait, DataBuffer, GraphicsContextTrait, RenderPassTrait};
use koi_assets::*;
use koi_transform::Transform;

pub struct Renderer {
    pub raw_graphics_context: kgraphics::GraphicsContext,
    render_pass_pool: Vec<RenderPass>,
    pub automatically_redraw: bool,
    pub(crate) shader_snippets: std::collections::HashMap<&'static str, &'static str>,
    color_space: kcolor::ColorSpace,
}

impl Renderer {
    pub(crate) fn new(
        raw_graphics_context: kgraphics::GraphicsContext,
        color_space: kgraphics::ColorSpace,
    ) -> Self {
        Self {
            raw_graphics_context,
            render_pass_pool: Vec::new(),
            automatically_redraw: true,
            shader_snippets: std::collections::HashMap::new(),
            color_space: match color_space {
                kgraphics::ColorSpace::SRGB => kcolor::color_spaces::ENCODED_SRGB,
                kgraphics::ColorSpace::DisplayP3 => kcolor::color_spaces::ENCODED_DISPLAY_P3,
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
            render_pass
        } else {
            RenderPass {
                meshes_to_draw: Vec::new(),
                local_to_world_matrices: Vec::new(),
                data_buffers_to_cleanup: Vec::new(),
                data_buffers_to_cleanup1: Vec::new(),
                camera: camera.clone(),
                camera_transform: *camera_transform,
                view_width,
                view_height,
                color_space: self.color_space.clone(),
                light_info: [LightInfo::default(); MAX_BOUND_LIGHTS],
                lights_bound: 0,
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
    ) {
        render_pass.execute(
            &mut self.raw_graphics_context,
            meshes,
            materials,
            shaders,
            textures,
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
    data_buffers_to_cleanup: Vec<DataBuffer<kmath::Mat4>>,
    data_buffers_to_cleanup1: Vec<DataBuffer<SceneInfoUniformBlock>>,
    color_space: kcolor::ColorSpace,
    light_info: [LightInfo; MAX_BOUND_LIGHTS],
    lights_bound: usize,
}

impl RenderPass {
    pub fn add_directional_light(
        &mut self,
        transform: &Transform,
        directional_light: &crate::DirectionalLight,
    ) {
        if self.lights_bound < MAX_BOUND_LIGHTS {
            let light_info = &mut self.light_info[self.lights_bound];
            light_info.position = transform.position;
            light_info.direction = transform.forward();
            light_info.world_to_light = transform.local_to_world().inversed();
            // TODO: Preexpose
            let light_color: kmath::Vec4 = directional_light.color.to_rgb_color(self.color_space);
            light_info.color_and_intensity =
                light_color.xyz() * directional_light.intensity_illuminance;
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
        graphics: &mut kgraphics::GraphicsContext,
        meshes: &AssetStore<Mesh>,
        materials: &AssetStore<Material>,
        shaders: &AssetStore<Shader>,
        textures: &AssetStore<Texture>,
    ) {
        // Sort meshes by material, then mesh.
        // TODO: This could be made more efficient by sorting by pipeline as well.
        // As-is small material variants will incur a cost.

        let mut command_buffer = graphics.new_command_buffer();

        let mut render_pass = command_buffer.begin_render_pass_with_framebuffer(
            &kgraphics::Framebuffer::default(),
            self.camera
                .clear_color
                .map(|v| v.to_rgb_color(self.color_space).into()),
        );
        render_pass.set_viewport(0, 0, self.view_width as u32, self.view_height as u32);

        let mut render_pass_executor = RenderPassExecutor {
            graphics,
            meshes,
            materials,
            shaders,
            textures,
            render_pass,
            local_to_world_matrices: &mut self.local_to_world_matrices,
            current_material_and_shader: None,
            current_gpu_mesh: None,
            camera_position: self.camera_transform.position,
            world_to_camera: self.camera_transform.local_to_world().inversed(),
            camera_to_screen: self
                .camera
                .projection_matrix(self.view_width, self.view_height),
            data_buffers_to_cleanup: &mut self.data_buffers_to_cleanup,
            data_buffers_to_cleanup1: &mut self.data_buffers_to_cleanup1,
            color_space: &self.color_space,
            lights_bound: 0,
            light_info: &mut self.light_info,
        };
        render_pass_executor.execute(&mut self.meshes_to_draw);
        command_buffer.present();
        graphics.commit_command_buffer(command_buffer);

        for data_buffer in self.data_buffers_to_cleanup.drain(..) {
            graphics.delete_data_buffer(data_buffer);
        }

        for data_buffer in self.data_buffers_to_cleanup1.drain(..) {
            graphics.delete_data_buffer(data_buffer);
        }
    }
}

struct RenderPassExecutor<'a> {
    graphics: &'a mut kgraphics::GraphicsContext,
    meshes: &'a AssetStore<Mesh>,
    materials: &'a AssetStore<Material>,
    shaders: &'a AssetStore<Shader>,
    textures: &'a AssetStore<Texture>,
    render_pass: kgraphics::RenderPass<'a>,
    local_to_world_matrices: &'a mut Vec<kmath::Mat4>,
    current_material_and_shader: Option<(&'a Material, &'a Shader)>,
    current_gpu_mesh: Option<&'a GPUMesh>,
    camera_position: kmath::Vec3,
    world_to_camera: kmath::Mat4,
    camera_to_screen: kmath::Mat4,
    data_buffers_to_cleanup: &'a mut Vec<DataBuffer<kmath::Mat4>>,
    data_buffers_to_cleanup1: &'a mut Vec<DataBuffer<SceneInfoUniformBlock>>,
    color_space: &'a ColorSpace,
    light_info: &'a mut [LightInfo; MAX_BOUND_LIGHTS],
    lights_bound: usize,
}

impl<'a> RenderPassExecutor<'a> {
    fn render_group(&mut self) {
        if let Some(gpu_mesh) = self.current_gpu_mesh {
            if let Some((material, shader)) = self.current_material_and_shader {
                self.render_pass.set_pipeline(&shader.pipeline);
                let shader_properties = &shader.shader_render_properties;

                // Bind the material properties
                {
                    let sp = &shader.shader_render_properties;
                    self.render_pass.set_vec4_property(
                        &sp.p_base_color,
                        material.base_color.to_rgb_color(*self.color_space).into(),
                    );
                    let texture_unit = 0;

                    self.render_pass.set_texture_property(
                        &sp.p_base_color_texture,
                        Some(material.base_color_texture.as_ref().map_or_else(
                            || &self.textures.get(&Texture::WHITE).0,
                            |t| &self.textures.get(t).0,
                        )),
                        texture_unit,
                    );
                    //texture_unit += 1;

                    self.render_pass
                        .set_float_property(&sp.p_metallic, material.metallicness);
                    self.render_pass
                        .set_float_property(&sp.p_roughness, material.roughness);
                    self.render_pass
                        .set_float_property(&sp.p_ambient, material.ambient_scale);
                    self.render_pass
                        .set_float_property(&sp.p_emissive, material.emissiveness);
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
                        self.render_pass.set_vertex_attribute_to_constant(
                            &shader_properties.color_attribute,
                            &[1.0, 1.0, 1.0, 1.0],
                        );
                    }
                }

                // Upload a new buffer for the thing being rendered.
                // TODO: Investigate if this is inefficient for single objects.
                let local_to_world_data = self
                    .graphics
                    .new_data_buffer(self.local_to_world_matrices)
                    .unwrap();

                self.render_pass.set_instance_attribute(
                    &shader_properties.local_to_world_instance_attribute,
                    Some(&local_to_world_data),
                );
                self.render_pass.draw_triangles_instanced(
                    gpu_mesh.index_end - gpu_mesh.index_start,
                    &gpu_mesh.index_buffer,
                    self.local_to_world_matrices.len() as u32,
                );
                self.local_to_world_matrices.clear();

                // This data buffer is deleted later after the commands are submitted.
                self.data_buffers_to_cleanup.push(local_to_world_data);
            }
        }
    }

    fn execute(&mut self, meshes_to_draw: &mut [(Handle<Material>, Handle<Mesh>, kmath::Mat4)]) {
        // Bind global data.
        {
            let data_buffer = self
                .graphics
                .new_data_buffer(&[SceneInfoUniformBlock {
                    p_world_to_camera: self.world_to_camera,
                    p_camera_to_screen: self.camera_to_screen,
                    p_camera_position: self.camera_position,
                    p_dither_scale: 1.0,
                    p_fog_start: 0.0,
                    p_fog_end: 100.0,
                    __padding: 0.0,
                    light_count: self.lights_bound as _,
                    // TODO: Don't do a clone here
                    lights: self.light_info.clone(),
                }])
                .unwrap();
            self.render_pass.set_uniform_block(
                &kgraphics::UniformBlock::from_location(0),
                Some(&data_buffer),
            );

            // TODO: Come up with a more elegant way to cleanup allocation buffers.
            self.data_buffers_to_cleanup1.push(data_buffer);
        }

        meshes_to_draw.sort_by_key(|v| (v.0.clone(), v.1.clone()));

        let mut current_mesh = None;
        let mut current_material = None;

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

            self.local_to_world_matrices.push(*local_to_world_matrix);
        }

        self.render_group();
    }
}
