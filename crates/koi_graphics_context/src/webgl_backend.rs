use crate::*;
use gl_shared::*;
use kwasm::*;

pub struct WebGLBackend {
    new_pipeline: JSObject,
    new_buffer: JSObject,
    new_texture: JSObject,
    update_texture: JSObject,
    destroy: JSObject,
    execute_commands: JSObject,
    generate_mip_maps: JSObject,
}

impl WebGLBackend {
    pub fn new(settings: GraphicsContextSettings) -> Self {
        let o = JSObjectFromString::new(include_str!("webgl_backend.js"));
        let setup = o.get_property("setup");

        let msaa_enabled = if settings.samples > 0 { 1 } else { 0 };
        setup.call_raw(&[msaa_enabled]);

        Self {
            new_pipeline: o.get_property("new_pipeline"),
            new_buffer: o.get_property("new_buffer"),
            new_texture: o.get_property("new_texture"),
            update_texture: o.get_property("update_texture"),
            destroy: o.get_property("destroy"),
            execute_commands: o.get_property("execute_commands"),
            generate_mip_maps: o.get_property("generate_mip_maps"),
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
            //self.generate_mip_map.call_raw(&[texture, TEXTURE_2D]);
        }
    }
}

impl backend_trait::BackendTrait for WebGLBackend {
    unsafe fn set_main_window(&mut self, window: &kapp::Window) {
        // Does nothing for now
    }

    unsafe fn execute_command_buffer(
        &mut self,
        command_buffer: &crate::CommandBuffer,
        buffer_sizes: &Vec<u32>,
        texture_sizes: &Vec<(u32, u32, u32)>,
    ) {
        todo!()
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
        todo!()
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
        todo!()
    }

    unsafe fn new_pipeline(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
        pipeline_settings: PipelineSettings,
    ) -> Result<PipelineInner, String> {
        let vertex_source = JSString::new(&vertex_source);
        let fragment_source = JSString::new(&fragment_source);
        let js_object = self
            .new_pipeline
            .call_2_arg(&vertex_source, &fragment_source)
            .ok_or_else(|| "Could not compile shader")?;

        Ok(crate::PipelineInner {
            program_index: js_object.leak(),
            pipeline_settings,
            uniforms: std::collections::HashMap::new(),
            uniform_blocks: Vec::new(),
            vertex_attributes: std::collections::HashMap::new(),
        })
    }

    unsafe fn delete_pipeline(&mut self, pipeline_inner: PipelineInner) {
        todo!()
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
        todo!()
    }
}
