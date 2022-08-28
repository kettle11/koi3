mod assets;
use std::collections::btree_map::Range;

use assets::*;

pub mod backend_trait;

mod command_buffer;
pub use command_buffer::*;

#[cfg(feature = "gl")]
mod gl_backend;
#[cfg(feature = "gl")]
mod gl_shared;

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

#[derive(Clone, Copy)]
pub struct PipelineSettings {
    pub faces_to_render: FacesToRender,
    pub blending: Option<(BlendFactor, BlendFactor)>,
    pub depth_test: DepthTest,
}

impl Default for PipelineSettings {
    fn default() -> Self {
        Self {
            faces_to_render: FacesToRender::Front,
            blending: None,
            depth_test: DepthTest::LessOrEqual,
        }
    }
}

#[derive(Clone)]
pub struct TextureInner {
    index: u32,
    texture_type: TextureType,
    mip: u8,
}

#[derive(Debug, Clone)]
enum TextureType {
    None,
    Texture,
    RenderBuffer,
    CubeMap { face: u8 },
    DefaultFramebuffer,
}

pub struct Texture(Handle<TextureInner>);

#[derive(Copy, Clone, Debug)]
pub enum FilterMode {
    Nearest,
    Linear,
}

#[derive(Copy, Clone, Debug)]
pub enum WrappingMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PixelFormat {
    R8Unorm,
    RG8Unorm,
    RGB8Unorm,
    RGBA8Unorm,
    Depth16,
    Depth24,
    Depth32F,
    RGBA16F,
    RGBA32F,
    // RGB16F,
    // RGB32F,
}

#[derive(Clone, Copy, Debug)]
pub enum FacesToRender {
    Front,
    Back,
    FrontAndBack,
    None,
}

#[derive(Clone, Copy, Debug)]
/// Specifies if a pixel will be rendered based on the z-buffer value.
pub enum DepthTest {
    /// Effectively disables depth testing.
    AlwaysPass,
    /// Write the pixel if its z value is less than the current value.
    Less,
    /// Write the pixel if its z value is greater than the current value.
    Greater,
    /// Write the pixel if its z value is less than or equal to the current value.
    LessOrEqual,
    /// Write the pixel if its z value is greater than or equal to the current value.s
    GreaterOrEqual,
}

/// This should be expanded
#[derive(Clone, Copy, Debug)]
pub enum BlendFactor {
    /// source_pixel
    One,
    /// source_pixel.alpha
    SourceAlpha,
    /// 1.0 - source_pixel.alpha
    OneMinusSourceAlpha,
}

#[derive(Copy, Clone, Debug)]
pub struct TextureSettings {
    pub srgb: bool,
    pub minification_filter: FilterMode,
    pub magnification_filter: FilterMode,
    /// How this texture is sampled between mipmaps.
    /// Defaults fo [FilterMode::Linear]
    pub mipmap_filter: FilterMode,
    pub generate_mipmaps: bool,
    pub wrapping_horizontal: WrappingMode,
    pub wrapping_vertical: WrappingMode,
    pub border_color: (f32, f32, f32, f32),
    pub msaa_samples: u8,
}

impl Default for TextureSettings {
    fn default() -> Self {
        Self {
            srgb: false,
            minification_filter: FilterMode::Nearest,
            magnification_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            generate_mipmaps: true,
            wrapping_horizontal: WrappingMode::Repeat,
            wrapping_vertical: WrappingMode::Repeat,
            border_color: (0., 0., 0., 1.),
            msaa_samples: 0,
        }
    }
}

#[derive(Clone)]
pub struct Pipeline(Handle<PipelineInner>);

#[derive(Clone)]
pub struct PipelineInner {
    program_index: u32,
    pipeline_settings: PipelineSettings,
}

pub struct CubeMap;

pub struct DataBuffer<D: GraphicsDataTrait> {
    phantom: std::marker::PhantomData<fn() -> D>,
}

pub struct TriangleBuffer(Handle<TriangleBufferInner>);

#[derive(Clone)]
pub struct TriangleBufferInner {
    index: u32,
}

pub trait GraphicsDataTrait {}

pub struct GraphicsContext {
    command_buffer_pool: Vec<CommandBuffer>,
    backend: Box<dyn backend_trait::BackendTrait>,
    texture_assets: Assets<TextureInner>,
    pipeline_assets: Assets<PipelineInner>,
    triangle_buffer_assets: Assets<TriangleBufferInner>,
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
                triangle_buffer_assets: Assets::new(),
            }
        }
    }

    pub fn cleanup(&mut self) {
        unsafe {
            for dropped_texture in self.texture_assets.get_dropped_assets() {
                self.backend.delete_texture(dropped_texture);
            }

            for dropped_pipeline in self.pipeline_assets.get_dropped_assets() {
                self.backend.delete_pipeline(dropped_pipeline);
            }

            for dropped_triangle_buffer in self.triangle_buffer_assets.get_dropped_assets() {
                self.backend.delete_triangle_buffer(dropped_triangle_buffer);
            }
        }
    }

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
    pub fn new_texture(
        &mut self,
        width: usize,
        height: usize,
        depth: usize,
        pixel_format: PixelFormat,
        settings: TextureSettings,
    ) -> Texture {
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

    pub fn update_texture(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        width: usize,
        height: usize,
        depth: usize,
        settings: TextureSettings,
    ) {
        todo!()
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

    pub fn new_data_buffer<D: GraphicsDataTrait>(&mut self, data: D) -> DataBuffer<D> {
        todo!()
    }

    pub fn new_triangle_buffer(&mut self, data: &[[u32; 3]]) -> TriangleBuffer {
        todo!()
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
            self.backend.execute_command_buffer(&command_buffer);
        }
        self.command_buffer_pool.push(command_buffer);
        self.cleanup();
    }
}
