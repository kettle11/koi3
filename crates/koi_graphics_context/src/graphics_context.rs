use core::slice;

use crate::*;

pub struct GraphicsContextSettings {
    /// If possible, should a high resolution framebuffer be requested?
    pub high_resolution_framebuffer: bool,
    /// How many MSAA samples should be requested for the framebuffer?
    pub samples: u8,
    // pub color_space: Option<ColorSpace>,
}

impl Default for GraphicsContextSettings {
    fn default() -> Self {
        Self {
            high_resolution_framebuffer: true,
            samples: 2,
            // color_space: Some(ColorSpace::SRGB),
        }
    }
}

pub struct GraphicsContext {
    command_buffer_pool: Vec<CommandBuffer>,
    backend: Box<dyn backend_trait::BackendTrait>,
    texture_assets: Assets<TextureInner>,
    pipeline_assets: Assets<PipelineInner>,
    buffer_assets: Assets<BufferInner>,
    buffer_sizes_bytes: Vec<u32>,
}

impl GraphicsContext {
    pub fn new(settings: GraphicsContextSettings, initial_window: &kapp::Window) -> Self {
        unsafe {
            Self {
                #[cfg(feature = "gl")]
                backend: Box::new(gl_backend::GLBackend::new(settings, initial_window)),
                command_buffer_pool: Vec::new(),
                texture_assets: Assets::new(),
                pipeline_assets: Assets::new(),
                buffer_assets: Assets::new(),
                buffer_sizes_bytes: Vec::new(),
            }
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

    /// Create a [Texture].
    /// After creating the [Texture] set its data with [Self::update_texture]
    pub fn new_texture<D: TextureDataTrait>(
        &mut self,
        width: usize,
        height: usize,
        depth: usize,
        settings: TextureSettings,
    ) -> Texture {
        let pixel_format = D::PIXEL_FORMAT;
        unsafe {
            Texture(self.texture_assets.new_handle(self.backend.new_texture(
                width,
                height,
                depth,
                pixel_format,
                settings,
            )))
        }
    }

    /// Update the contents of a [Texture]
    pub fn update_texture<D: TextureDataTrait>(
        &mut self,
        texture: &Texture,
        x: usize,
        y: usize,
        z: usize,
        width: usize,
        height: usize,
        depth: usize,
        data: &[D],
        settings: TextureSettings,
    ) {
        assert_eq!(
            data.len(),
            width * height * depth,
            "Data passed in does not match the size being updated"
        );

        // TODO: Assert that the texture is actually this size.
        
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

    pub fn new_cube_map(
        &mut self,
        width: usize,
        height: usize,
        depth: usize,
        settings: TextureSettings,
    ) -> CubeMap {
        todo!()
    }

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
            self.backend
                .execute_command_buffer(&command_buffer, &self.buffer_sizes_bytes);
        }
        self.command_buffer_pool.push(command_buffer);
        self.cleanup();
    }
}