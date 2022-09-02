use crate::*;

pub struct GraphicsContextSettings {
    /// If possible, should a high resolution framebuffer be requested?
    pub high_resolution_framebuffer: bool,
    /// How many MSAA samples should be requested for the framebuffer?
    pub samples: u8,
    pub color_space: Option<ColorSpace>,
}

impl Default for GraphicsContextSettings {
    fn default() -> Self {
        Self {
            high_resolution_framebuffer: true,
            samples: 2,
            color_space: Some(ColorSpace::SRGB),
        }
    }
}

pub struct GraphicsContext {
    command_buffer_pool: Vec<CommandBuffer>,
    backend: Box<dyn backend_trait::BackendTrait>,
    texture_assets: Assets<TextureInner>,
    pipeline_assets: Assets<PipelineInner>,
    buffer_assets: Assets<BufferInner>,
    cube_map_assets: Assets<CubeMapInner>,
    buffer_sizes_bytes: Vec<u32>,
    texture_size_pixels: Vec<(u32, u32, u32)>,
}

impl GraphicsContext {
    pub async fn new(settings: GraphicsContextSettings) -> Self {
        Self {
            #[cfg(all(target_arch = "wasm32", feature = "webgl"))]
            backend: Box::new(unsafe { webgl_backend::WebGLBackend::new(settings) }),
            #[cfg(all(feature = "gl", not(target_arch = "wasm32")))]
            backend: Box::new(unsafe { gl_backend::GLBackend::new(settings) }),
            #[cfg(feature = "webgpu")]
            backend: Box::new(webgpu_backend::WebGPUBackend::new(settings).await.unwrap()),
            command_buffer_pool: Vec::new(),
            texture_assets: Assets::new(),
            pipeline_assets: Assets::new(),
            buffer_assets: Assets::new(),
            cube_map_assets: Assets::new(),
            buffer_sizes_bytes: Vec::new(),
            texture_size_pixels: Vec::new(),
        }
    }

    pub fn set_main_window(&mut self, window: &kapp::Window) {
        unsafe {
            self.backend.set_main_window(window);
        }
    }

    /// Free memory for unused resources.
    /// Called automatically after each CommandBuffer is executed.
    pub fn cleanup(&mut self) {
        unsafe {
            for dropped_texture in self.texture_assets.get_dropped_assets() {
                self.backend.delete_texture(dropped_texture);
            }

            for dropped_pipeline in self.pipeline_assets.get_dropped_assets() {
                self.backend.delete_pipeline(dropped_pipeline);
            }

            for dropped_buffer in self.buffer_assets.get_dropped_assets() {
                self.backend.delete_buffer(dropped_buffer);
            }

            for dropped_cube_map in self.cube_map_assets.get_dropped_assets() {
                self.backend.delete_cube_map(dropped_cube_map);
            }
        }
    }

    /// Creates a new [Pipeline] that can be used for rendering.
    pub fn new_pipeline(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
        pipeline_settings: PipelineSettings,
    ) -> Result<Pipeline, String> {
        unsafe {
            Ok(Pipeline(
                self.pipeline_assets.new_handle(self.backend.new_pipeline(
                    vertex_source,
                    fragment_source,
                    pipeline_settings,
                )?),
            ))
        }
    }

    #[inline]
    pub fn new_texture_with_data<D: TextureDataTrait>(
        &mut self,
        width: u32,
        height: u32,
        depth: u32,
        data: &[D],
        settings: TextureSettings,
    ) -> Texture {
        let texture = self.new_texture::<D>(width, height, depth, settings);
        self.update_texture(&texture, 0, 0, 0, width, height, depth, data, settings);
        texture
    }

    /// Create a [Texture].
    /// After creating the [Texture] set its data with [Self::update_texture]
    #[inline]
    pub fn new_texture<D: TextureDataTrait>(
        &mut self,
        width: u32,
        height: u32,
        depth: u32,
        settings: TextureSettings,
    ) -> Texture {
        let pixel_format = D::PIXEL_FORMAT;
        self.new_texture_with_pixel_format(width, height, depth, pixel_format, settings)
    }

    pub fn new_texture_with_pixel_format(
        &mut self,
        width: u32,
        height: u32,
        depth: u32,
        pixel_format: PixelFormat,
        settings: TextureSettings,
    ) -> Texture {
        let handle = unsafe {
            self.texture_assets.new_handle(self.backend.new_texture(
                width,
                height,
                depth,
                pixel_format,
                settings,
            ))
        };
        self.texture_size_pixels.resize(
            self.texture_size_pixels
                .len()
                .max(handle.inner().index as usize + 1),
            (0, 0, 0),
        );
        self.texture_size_pixels[handle.inner().index as usize] = (width, height, depth);
        Texture(handle)
    }

    pub unsafe fn new_texture_with_bytes(
        &mut self,
        width: u32,
        height: u32,
        depth: u32,
        data: &[u8],
        pixel_format: PixelFormat,
        settings: TextureSettings,
    ) -> Texture {
        let texture =
            self.new_texture_with_pixel_format(width, height, depth, pixel_format, settings);

        let data = unsafe { slice_to_bytes(data) };
        unsafe {
            self.backend.update_texture(
                &texture.0.inner(),
                0,
                0,
                0,
                width,
                height,
                depth,
                data,
                settings,
            )
        }
        texture
    }
    /// Update the contents of a [Texture]
    #[inline]
    pub fn update_texture<D: TextureDataTrait>(
        &mut self,
        texture: &Texture,
        x: u32,
        y: u32,
        z: u32,
        width: u32,
        height: u32,
        depth: u32,
        data: &[D],
        settings: TextureSettings,
    ) {
        assert_eq!(
            data.len() as u32,
            width * height * depth,
            "Data passed in does not match the size being updated"
        );

        assert_eq!(
            D::PIXEL_FORMAT,
            texture.0.inner().pixel_format,
            "This texture's pixels are stored in a different PixelFormat"
        );

        // Assert that the data passed in isn't larger than the texture.
        {
            let (w, h, d) = self.texture_size_pixels[texture.0.inner().index as usize];
            assert!(x < w);
            assert!(y < h);
            assert!(z < d);
            assert!(x + width <= w);
            assert!(y + height <= h);
            assert!(z + depth <= d);
        }

        let data = unsafe { slice_to_bytes(data) };
        unsafe {
            self.backend.update_texture(
                &texture.0.inner(),
                x,
                y,
                z,
                width,
                height,
                depth,
                data,
                settings,
            )
        }
    }

    #[inline]
    pub fn new_cube_map_with_data<D: TextureDataTrait>(
        &mut self,
        width: u32,
        height: u32,
        data: &[&[D]; 6],
        settings: TextureSettings,
    ) -> CubeMap {
        for face in data {
            assert!(face.len() as u32 == width * height);
        }
        let pixel_format = D::PIXEL_FORMAT;
        let cube_map = unsafe {
            self.cube_map_assets.new_handle(self.backend.new_cube_map(
                width,
                height,
                pixel_format,
                settings,
            ))
        };
        let cube_map = CubeMap(cube_map);
        self.update_cube_map(&cube_map, width, height, data, settings);
        cube_map
    }

    #[inline]
    pub fn update_cube_map<D: TextureDataTrait>(
        &mut self,
        cube_map: &CubeMap,
        width: u32,
        height: u32,
        data: &[&[D]; 6],
        settings: TextureSettings,
    ) {
        let data = unsafe {
            [
                slice_to_bytes(data[0]),
                slice_to_bytes(data[1]),
                slice_to_bytes(data[2]),
                slice_to_bytes(data[3]),
                slice_to_bytes(data[4]),
                slice_to_bytes(data[5]),
            ]
        };

        unsafe {
            self.backend
                .update_cube_map(&cube_map.0.inner(), width, height, &data, settings)
        }
    }

    #[inline]
    pub fn new_buffer<D: BufferDataTrait>(
        &mut self,
        data: &[D],
        buffer_usage: BufferUsage,
    ) -> Buffer<D> {
        let data_bytes = unsafe { slice_to_bytes(data) };
        let handle = self
            .buffer_assets
            .new_handle(unsafe { self.backend.new_buffer(buffer_usage, data_bytes) });
        self.buffer_sizes_bytes.resize(
            (handle.inner().index as usize + 1).max(self.buffer_sizes_bytes.len()),
            0,
        );
        self.buffer_sizes_bytes[handle.inner().index as usize] = data_bytes.len() as u32;
        Buffer {
            handle,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn new_command_buffer(&mut self) -> CommandBuffer {
        let mut command_buffer = self
            .command_buffer_pool
            .pop()
            .unwrap_or_else(|| CommandBuffer::new());
        command_buffer.clear();
        command_buffer
    }

    pub fn execute_command_buffer(&mut self, command_buffer: CommandBuffer) {
        unsafe {
            self.backend.execute_command_buffer(
                &command_buffer,
                &self.buffer_sizes_bytes,
                &self.texture_size_pixels,
            );
        }
        self.command_buffer_pool.push(command_buffer);
        self.cleanup();
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new_texture_from_js_object(
        &mut self,
        width: u32,
        height: u32,
        js_object_data: &kwasm::JSObjectDynamic,
        pixel_format: PixelFormat,
        texture_settings: TextureSettings,
    ) -> Texture {
        unsafe {
            Texture(
                self.texture_assets
                    .new_handle(self.backend.new_texture_from_js_object(
                        width,
                        height,
                        js_object_data,
                        pixel_format,
                        texture_settings,
                    )),
            )
        }
    }
}
