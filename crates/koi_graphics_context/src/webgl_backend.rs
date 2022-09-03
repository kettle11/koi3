use crate::*;
use gl_shared::*;
use kwasm::*;

pub struct WebGLBackend {
    new_pipeline: JSObject,
    new_buffer: JSObject,
    new_texture: JSObject,
    update_texture: JSObject,
    generate_mip_maps: JSObject,
    clear: JSObject,
    viewport: JSObject,
    set_pipeline: JSObject,
    set_uniform_block: JSObject,
    set_uniform_float: JSObject,
    set_uniform_int: JSObject,
    set_texture: JSObject,
    set_attribute: JSObject,
    set_attribute_to_constant: JSObject,
    draw: JSObject,
    delete_buffer: JSObject,
    delete_texture: JSObject,
    delete_program: JSObject,
    get_uniform_name_and_type: JSObject,
    get_uniform_location: JSObject,
    get_program_parameter: JSObject,
    get_attribute_name_and_type: JSObject,
    get_attribute_location: JSObject,
    get_uniform_block_name_and_size: JSObject,
    uniform_block_binding: JSObject,
    use_program: JSObject,
}

impl WebGLBackend {
    pub unsafe fn new(settings: GraphicsContextSettings) -> Self {
        let o = JSObjectFromString::new(include_str!("webgl_backend.js"));
        let setup = o.get_property("setup");

        let msaa_enabled = if settings.samples > 0 { 1 } else { 0 };
        setup.call_raw(&[msaa_enabled]);

        Self {
            new_pipeline: o.get_property("new_pipeline"),
            new_buffer: o.get_property("new_buffer"),
            new_texture: o.get_property("new_texture"),
            update_texture: o.get_property("update_texture"),
            generate_mip_maps: o.get_property("generate_mip_maps"),
            clear: o.get_property("clear"),
            viewport: o.get_property("viewport"),
            set_pipeline: o.get_property("set_pipeline"),
            set_uniform_block: o.get_property("set_uniform_block"),
            set_uniform_float: o.get_property("set_uniform_float"),
            set_uniform_int: o.get_property("set_uniform_int"),
            set_texture: o.get_property("set_texture"),
            set_attribute: o.get_property("set_attribute"),
            draw: o.get_property("draw"),
            delete_buffer: o.get_property("delete_buffer"),
            delete_texture: o.get_property("delete_texture"),
            delete_program: o.get_property("delete_program"),
            get_uniform_name_and_type: o.get_property("get_uniform_name_and_type"),
            get_uniform_location: o.get_property("get_uniform_location"),
            get_program_parameter: o.get_property("get_program_parameter"),
            get_attribute_name_and_type: o.get_property("get_attribute_name_and_type"),
            get_attribute_location: o.get_property("get_attribute_location"),
            get_uniform_block_name_and_size: o.get_property("get_uniform_block_name_and_size"),
            uniform_block_binding: o.get_property("uniform_block_binding"),
            set_attribute_to_constant: o.get_property("set_attribute_to_constant"),
            use_program: o.get_property("use_program"),
        }
    }

    fn update_texture_internal(
        &mut self,
        texture: &TextureInner,
        width: u32,
        height: u32,
        depth: u32,
        js_object_data: &kwasm::JSObject,
        data: &[u8],
        pixel_format: PixelFormat,
        texture_settings: TextureSettings,
    ) {
        let (pixel_format, inner_pixel_format, type_) =
            crate::gl_shared::pixel_format_to_gl_format_and_inner_format_and_type(
                pixel_format,
                texture_settings.srgb,
            );

        let (target, texture) = match &texture.texture_type {
            TextureType::Texture => (TEXTURE_2D, texture.index),
            _ => todo!(),
        };

        let minification_filter = minification_filter_to_gl_enum(
            texture_settings.minification_filter,
            texture_settings.mipmap_filter,
            texture_settings.generate_mipmaps,
        );
        let magnification_filter =
            magnification_filter_to_gl_enum(texture_settings.magnification_filter);

        let wrapping_horizontal = wrapping_to_gl_enum(texture_settings.wrapping_horizontal);
        let wrapping_vertical = wrapping_to_gl_enum(texture_settings.wrapping_vertical);

        let (data_ptr, data_len) = (data.as_ptr() as u32, data.len() as u32);

        self.update_texture.call_raw(&[
            texture,
            target,
            target,
            inner_pixel_format,
            width,
            height,
            depth,
            pixel_format,
            type_,
            js_object_data.index(),
            data_ptr,
            data_len,
            minification_filter,
            magnification_filter,
            wrapping_horizontal,
            wrapping_vertical,
        ]);

        if texture_settings.generate_mipmaps {
            self.generate_mip_maps.call_raw(&[texture, TEXTURE_2D]);
        }
    }
}

impl backend_trait::BackendTrait for WebGLBackend {
    unsafe fn set_main_window(&mut self, _window: &kapp::Window) {
        // Does nothing for now
    }

    unsafe fn execute_command_buffer(
        &mut self,
        command_buffer: &crate::CommandBuffer,
        _buffer_sizes: &Vec<u32>,
        texture_sizes: &Vec<(u32, u32, u32)>,
    ) {
        // In the future this could be made more efficient by changing how Commandbuffer
        // is represented so it can be sent directly to the JS side.
        for command in command_buffer.commands.iter() {
            match command {
                Command::Present => {}
                Command::BeginRenderPass { clear_color } => {
                    let data_slice = clear_color.as_slice();
                    self.clear.call_raw(&[
                        data_slice.as_ptr() as _,
                        //(data_slice.len() * std::mem::size_of::<f32>()) as _,
                    ]);
                }
                Command::SetPipeline {
                    pipeline_index,
                    pipeline_settings,
                } => {
                    fn blending_to_gl(blending: BlendFactor) -> GLenum {
                        match blending {
                            BlendFactor::One => ONE,
                            BlendFactor::OneMinusSourceAlpha => ONE_MINUS_SRC_ALPHA,
                            BlendFactor::SourceAlpha => SRC_ALPHA,
                        }
                    }

                    let (source_blend_factor, destination_blend_factor) =
                        match pipeline_settings.blending {
                            Some((source_blend_factor, destination_blend_factor)) => (
                                blending_to_gl(source_blend_factor),
                                blending_to_gl(destination_blend_factor),
                            ),
                            None => (0, 0),
                        };

                    self.set_pipeline.call_raw(&[
                        *pipeline_index,
                        match pipeline_settings.depth_test {
                            DepthTest::AlwaysPass => ALWAYS,
                            DepthTest::Less => LESS,
                            DepthTest::Greater => GREATER,
                            DepthTest::LessOrEqual => LEQUAL,
                            DepthTest::GreaterOrEqual => GEQUAL,
                        },
                        match pipeline_settings.faces_to_render {
                            FacesToRender::Front => BACK,
                            FacesToRender::Back => FRONT,
                            FacesToRender::FrontAndBack => 0,
                            FacesToRender::None => FRONT_AND_BACK,
                        },
                        source_blend_factor,
                        destination_blend_factor,
                    ]);
                }
                Command::Draw {
                    index_buffer_index,
                    triangle_range,
                    instances,
                } => {
                    let count = triangle_range.end - triangle_range.start;
                    let count_vertices = count * 3;
                    let offset_bytes = triangle_range.start * 3 * std::mem::size_of::<u32>() as u32;
                    self.draw.call_raw(&[
                        index_buffer_index.unwrap_or(0),
                        count_vertices,
                        offset_bytes,
                        *instances,
                    ]);
                }
                &Command::SetViewPort {
                    x,
                    y,
                    width,
                    height,
                } => {
                    let data = [x, y, width, height];
                    let data_slice = data.as_slice();
                    self.viewport.call_raw(&[
                        data_slice.as_ptr() as _,
                        //(data_slice.len() * std::mem::size_of::<f32>()) as _,
                    ]);
                }
                Command::SetUniform {
                    uniform_info,
                    bump_handle,
                } => {
                    if let Some(location) = uniform_info.location {
                        let data = command_buffer.bump_allocator.get_raw_bytes(*bump_handle);
                        let (data_ptr, data_len) = (data.as_ptr() as u32, data.len() as u32);

                        match uniform_info.uniform_type {
                            UniformType::UInt(_) => todo!(),
                            UniformType::Int(n) => self
                                .set_uniform_int
                                .call_raw(&[1, location, n as u32, data_ptr, data_len]),
                            UniformType::Float(_) => self
                                .set_uniform_float
                                .call_raw(&[1, location, data_ptr, data_len]),
                            UniformType::Vec2(_) => self
                                .set_uniform_float
                                .call_raw(&[2, location, data_ptr, data_len]),
                            UniformType::Vec3(_) => self
                                .set_uniform_float
                                .call_raw(&[3, location, data_ptr, data_len]),
                            UniformType::Vec4(_) => self
                                .set_uniform_float
                                .call_raw(&[4, location, data_ptr, data_len]),
                            UniformType::Mat4(_) => self
                                .set_uniform_float
                                .call_raw(&[16, location, data_ptr, data_len]),
                            UniformType::Sampler2d
                            | UniformType::Sampler3d
                            | UniformType::SamplerCube => self
                                .set_uniform_int
                                .call_raw(&[1, location, 1, data_ptr, data_len]),
                        };
                    }
                }
                Command::SetUniformBlock {
                    uniform_block_index,
                    buffer,
                } => {
                    self.set_uniform_block.call_raw(&[
                        *uniform_block_index as u32,
                        buffer.as_ref().map_or(0, |b| b.handle.inner().index),
                    ]);
                }
                Command::SetAttribute {
                    attribute,
                    buffer,
                    per_instance,
                } => {
                    if let Some(info) = &attribute.info {
                        // TODO: This shouldn't only be supported for floats.
                        self.set_attribute.call_raw(&[
                            info.location,
                            info.byte_size / 4, // Number of components
                            buffer.as_ref().map_or(0, |b| b.handle.inner().index),
                            if *per_instance { 1 } else { 0 },
                        ]);
                    }
                }
                Command::SetAttributeToConstant { attribute, value } => {
                    if let Some(attribute_index) = attribute.info.as_ref() {
                        self.set_attribute_to_constant.call_raw(&[
                            attribute_index.location,
                            value.as_ptr() as u32,
                            // Be careful in the future with this. The other side expects
                            // the number of f32s, but that could change if this is made to accept non-floats
                            value.len() as u32,
                        ]);
                    }
                }
                Command::SetTexture {
                    texture_unit,
                    texture_index,
                } => {
                    let is_3d = texture_sizes[*texture_index as usize].2 > 1;

                    self.set_texture.call_raw(&[
                        TEXTURE0 + *texture_unit as u32,
                        if is_3d { TEXTURE_3D } else { TEXTURE_2D },
                        *texture_index,
                    ]);
                }
                Command::SetCubeMap {
                    texture_unit,
                    cube_map_index,
                } => {
                    self.set_texture.call_raw(&[
                        TEXTURE0 + *texture_unit as u32,
                        TEXTURE_CUBE_MAP,
                        *cube_map_index,
                    ]);
                }
            }
        }
    }

    unsafe fn new_texture(
        &mut self,
        _width: u32,
        _height: u32,
        _depth: u32,
        pixel_format: PixelFormat,
        _settings: TextureSettings,
    ) -> TextureInner {
        // TODO: Support other texture types
        // TODO: Consider initializing texture with a specific type.
        TextureInner {
            index: self.new_texture.call().unwrap().leak(),
            texture_type: TextureType::Texture,
            mip: 0,
            pixel_format,
        }
    }

    unsafe fn update_texture(
        &mut self,
        texture: &TextureInner,
        x: u32,
        y: u32,
        z: u32,
        width: u32,
        height: u32,
        depth: u32,
        data: &[u8],
        settings: TextureSettings,
    ) {
        if x > 0 || y > 0 || z > 0 || depth > 1 {
            todo!()
        }

        self.update_texture_internal(
            texture,
            width,
            height,
            depth,
            &JSObject::null(),
            data,
            texture.pixel_format,
            settings,
        )
    }

    unsafe fn delete_texture(&mut self, texture_inner: TextureInner) {
        (self.delete_buffer).call_1_arg(&JSObject::new_raw(texture_inner.index));
    }

    unsafe fn new_cube_map(
        &mut self,
        _width: u32,
        _height: u32,
        pixel_format: PixelFormat,
        _texture_settings: TextureSettings,
    ) -> CubeMapInner {
        CubeMapInner {
            index: self.new_texture.call().unwrap().leak(),
            pixel_format,
        }
    }

    unsafe fn update_cube_map(
        &mut self,
        cube_map: &CubeMapInner,
        width: u32,
        height: u32,
        data: &[&[u8]; 6],
        texture_settings: TextureSettings,
    ) {
        // Convert data to linear instead of sRGB if needed and flip the image vertically.
        let (pixel_format, inner_pixel_format, type_) =
            crate::gl_shared::pixel_format_to_gl_format_and_inner_format_and_type(
                cube_map.pixel_format,
                texture_settings.srgb,
            );

        let minification_filter = minification_filter_to_gl_enum(
            texture_settings.minification_filter,
            texture_settings.mipmap_filter,
            texture_settings.generate_mipmaps,
        );
        let magnification_filter =
            magnification_filter_to_gl_enum(texture_settings.magnification_filter);

        let wrapping_horizontal = wrapping_to_gl_enum(texture_settings.wrapping_horizontal);
        let wrapping_vertical = wrapping_to_gl_enum(texture_settings.wrapping_vertical);

        for i in 0..6 {
            let (data_ptr, data_len) = (data[i].as_ptr() as u32, data[i].len() as u32);

            self.update_texture.call_raw(&[
                cube_map.index,
                TEXTURE_CUBE_MAP,
                TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                inner_pixel_format,
                width,
                height,
                1,
                pixel_format,
                type_,
                0,
                data_ptr,
                data_len,
                minification_filter,
                magnification_filter,
                wrapping_horizontal,
                wrapping_vertical,
            ]);
        }
        if texture_settings.generate_mipmaps {
            self.generate_mip_maps
                .call_raw(&[cube_map.index, TEXTURE_CUBE_MAP]);
        }
    }

    unsafe fn delete_cube_map(&mut self, cube_map_inner: CubeMapInner) {
        (self.delete_texture).call_1_arg(&JSObject::new_raw(cube_map_inner.index));
    }

    unsafe fn new_pipeline(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
        pipeline_settings: PipelineSettings,
    ) -> Result<PipelineInner, String> {
        let vertex_source = JSString::new(&vertex_source);
        let fragment_source = JSString::new(&fragment_source);
        let program = self
            .new_pipeline
            .call_2_arg(&vertex_source, &fragment_source)
            .ok_or_else(|| "Could not compile shader")?;

        fn get_id(name: &str) -> Option<u32> {
            Some(name[2..name.find('_')?].parse().ok()?)
        }

        self.use_program.call_1_arg(&program);

        let mut uniforms = std::collections::HashMap::new();
        {
            let uniform_count = self
                .get_program_parameter
                .call_raw(&[program.index(), ACTIVE_UNIFORMS])
                .unwrap()
                .get_value_u32();

            for i in 0..uniform_count {
                let uniform_type = self
                    .get_uniform_name_and_type
                    .call_raw(&[program.index(), i])
                    .unwrap()
                    .get_value_u32();

                // TODO: This assertion about this not being an array is incorrect
                let uniform_type = gl_uniform_type_to_uniform_type(uniform_type, 1);
                let uniform_name = kwasm::get_string_from_host();

                // Passing the name immediately back to JS probably isn't the best here.
                if let Some(uniform_location) = self
                    .get_uniform_location
                    .call_2_arg(&program, &JSString::new(&uniform_name))
                {
                    match uniform_type {
                        UniformType::Sampler2d
                        | UniformType::Sampler3d
                        | UniformType::SamplerCube => {
                            // Bind the location once
                            let id = get_id(&uniform_name).expect(&&uniform_name);

                            let data = &[id];
                            let data_ptr = data.as_ptr();
                            self.set_uniform_int.call_raw(&[
                                1,
                                uniform_location.index(),
                                1,
                                data_ptr as u32,
                                (data.len() * std::mem::size_of::<u32>()) as u32,
                            ]);
                        }
                        _ => {}
                    }

                    uniforms.insert(
                        uniform_name,
                        UniformInfo {
                            pipeline_index: program.index(),
                            uniform_type,
                            location: Some(uniform_location.leak()),
                        },
                    );
                }
            }
        }

        let mut uniform_blocks = Vec::new();
        {
            let uniform_block_count = self
                .get_program_parameter
                .call_raw(&[program.index(), ACTIVE_UNIFORM_BLOCKS])
                .unwrap()
                .get_value_u32();

            for i in 0..uniform_block_count {
                let size_bytes = self
                    .get_uniform_block_name_and_size
                    .call_raw(&[program.index(), i])
                    .unwrap()
                    .get_value_u32();

                let name = kwasm::get_string_from_host();

                let binding_location = get_id(&name).ok_or_else(|| {
                        "Uniform blocks must be formatted with ub[binding_index]_name. EX: ub0_scene_info."
                    })?;

                (self.uniform_block_binding).call_raw(&[program.index(), i, binding_location]);
                uniform_blocks.push(UniformBlockInfo {
                    size_bytes,
                    location: i,
                });
            }
        }

        let mut vertex_attributes = std::collections::HashMap::new();
        {
            let vertex_attribute_count = self
                .get_program_parameter
                .call_raw(&[program.index(), ACTIVE_ATTRIBUTES])
                .unwrap()
                .get_value_u32();

            for i in 0..vertex_attribute_count {
                let attribute_type = self
                    .get_attribute_name_and_type
                    .call_raw(&[program.index(), i])
                    .unwrap()
                    .get_value_u32();

                let byte_size = match attribute_type {
                    FLOAT => 4,
                    FLOAT_VEC2 => 8,
                    FLOAT_VEC3 => 12,
                    FLOAT_VEC4 => 16,
                    FLOAT_MAT4 => 64,
                    _ => continue,
                };

                let attribute_name = kwasm::get_string_from_host();

                // Passing the name immediately back to JS probably isn't the best here.
                // Notably the attribute location index is *not* the index passed into `GetActiveAttrib`
                if let Some(attribute_location) = self
                    .get_attribute_location
                    .call_2_arg(&program, &JSString::new(&attribute_name))
                {
                    let location = attribute_location.get_value_u32();

                    vertex_attributes.insert(
                        attribute_name,
                        VertexAttributeInfo {
                            location,
                            byte_size,
                        },
                    );
                }
            }
        }

        Ok(crate::PipelineInner {
            program_index: program.leak(),
            pipeline_settings,
            uniforms,
            uniform_blocks,
            vertex_attributes,
        })
    }

    unsafe fn delete_pipeline(&mut self, pipeline_inner: PipelineInner) {
        (self.delete_program).call_1_arg(&JSObject::new_raw(pipeline_inner.program_index));
    }

    unsafe fn new_buffer(&mut self, buffer_usage: BufferUsage, data: &[u8]) -> BufferInner {
        let gl_buffer_usage = match buffer_usage {
            BufferUsage::Data => ARRAY_BUFFER,
            BufferUsage::Index => ELEMENT_ARRAY_BUFFER,
        };

        let (data_ptr, data_len) = (data.as_ptr() as u32, data.len() as u32);
        let js_object = self
            .new_buffer
            .call_raw(&[data_ptr as u32, data_len as u32, gl_buffer_usage])
            .unwrap();

        BufferInner {
            index: js_object.leak(),
            buffer_usage,
        }
    }

    unsafe fn delete_buffer(&mut self, buffer_inner: BufferInner) {
        (self.delete_buffer).call_1_arg(&JSObject::new_raw(buffer_inner.index));
    }
}
