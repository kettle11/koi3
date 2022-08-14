use crate::*;
use kgraphics::{CommandBufferTrait, DataBuffer, GraphicsContextTrait, RenderPassTrait};
use koi_assets::*;
use koi_transform::Transform;

pub struct Renderer {
    pub raw_graphics_context: kgraphics::GraphicsContext,
    render_pass_pool: Vec<RenderPass>,
    pub automatically_redraw: bool,
    pub(crate) shader_snippets: std::collections::HashMap<&'static str, &'static str>,
}

impl Renderer {
    pub(crate) fn new(raw_graphics_context: kgraphics::GraphicsContext) -> Self {
        Self {
            raw_graphics_context,
            render_pass_pool: Vec::new(),
            automatically_redraw: true,
            shader_snippets: std::collections::HashMap::new(),
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
            render_pass.camera_transform = camera_transform.clone();
            render_pass.meshes_to_draw.clear();
            render_pass.local_to_world_matrices.clear();
            render_pass.view_width = view_width;
            render_pass.view_height = view_height;
            render_pass.camera = camera.clone();
            render_pass.camera_transform = camera_transform.clone();
            render_pass
        } else {
            RenderPass {
                meshes_to_draw: Vec::new(),
                local_to_world_matrices: Vec::new(),
                data_buffers_to_cleanup: Vec::new(),
                camera: camera.clone(),
                camera_transform: camera_transform.clone(),
                view_width,
                view_height,
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
}

impl RenderPass {
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
            self.camera.clear_color.map(|v| v.to_linear_srgb().into()),
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
            world_to_camera: self.camera_transform.local_to_world().inversed(),
            // TODO: This projection matrix doesn't account for window dimensions.
            camera_to_screen: self
                .camera
                .projection_matrix(self.view_width, self.view_height),
            data_buffers_to_cleanup: &mut &mut self.data_buffers_to_cleanup,
        };
        render_pass_executor.execute(&mut self.meshes_to_draw);
        command_buffer.present();
        graphics.commit_command_buffer(command_buffer);

        for data_buffer in self.data_buffers_to_cleanup.drain(..) {
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
    world_to_camera: kmath::Mat4,
    camera_to_screen: kmath::Mat4,
    data_buffers_to_cleanup: &'a mut Vec<DataBuffer<kmath::Mat4>>,
}

impl<'a> RenderPassExecutor<'a> {
    fn render_group(&mut self) {
        if let Some(gpu_mesh) = self.current_gpu_mesh {
            if let Some((material, shader)) = self.current_material_and_shader {
                self.render_pass.set_pipeline(&shader.pipeline);
                let shader_properties = &shader.shader_render_properties;

                // TODO: Use uniform buffer objects instead to avoid needing to rebind this data
                // on each material change.
                {
                    self.render_pass.set_mat4_property(
                        &shader_properties.world_to_camera,
                        self.world_to_camera.as_array(),
                    );
                    self.render_pass.set_mat4_property(
                        &shader_properties.camera_to_screen,
                        self.camera_to_screen.as_array(),
                    );
                }

                // Bind the material properties
                {
                    self.render_pass.set_vec4_property(
                        &shader.shader_render_properties.p_base_color,
                        material.base_color.to_linear_srgb().into(),
                    );
                    let texture_unit = 0;

                    self.render_pass.set_texture_property(
                        &shader.shader_render_properties.p_base_color_texture,
                        Some(material.base_color_texture.as_ref().map_or_else(
                            || &self.textures.get(&Texture::WHITE).0,
                            |t| &self.textures.get(t).0,
                        )),
                        texture_unit,
                    );
                    //texture_unit += 1;
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
                    .new_data_buffer(&self.local_to_world_matrices)
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
