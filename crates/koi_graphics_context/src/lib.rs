mod assets;

use assets::*;

pub mod backend_trait;

mod command_buffer;
pub use command_buffer::*;

#[cfg(feature = "gl")]
mod gl_backend;
#[cfg(feature = "gl")]
mod gl_shared;

mod graphics_context;
pub use graphics_context::*;

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
    pixel_format: PixelFormat,
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

pub trait TextureDataTrait {
    const PIXEL_FORMAT: PixelFormat;
}

impl TextureDataTrait for u8 {
    const PIXEL_FORMAT: PixelFormat = PixelFormat::R8Unorm;
}
impl TextureDataTrait for [u8; 2] {
    const PIXEL_FORMAT: PixelFormat = PixelFormat::RG8Unorm;
}
impl TextureDataTrait for [u8; 3] {
    const PIXEL_FORMAT: PixelFormat = PixelFormat::RGB8Unorm;
}
impl TextureDataTrait for [u8; 4] {
    const PIXEL_FORMAT: PixelFormat = PixelFormat::RGBA8Unorm;
}
impl TextureDataTrait for [f32; 4] {
    const PIXEL_FORMAT: PixelFormat = PixelFormat::RGBA32F;
}

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
    pub(crate) program_index: u32,
    pub(crate) pipeline_settings: PipelineSettings,
    pub(crate) uniforms: std::collections::HashMap<String, UniformInfo>,
    pub(crate) uniform_blocks: Vec<UniformBlockInfo>,
    pub(crate) vertex_attributes: std::collections::HashMap<String, VertexAttributeInfo>,
}

pub struct CubeMap;

pub struct DataBuffer<D: GraphicsDataTrait> {
    phantom: std::marker::PhantomData<fn() -> D>,
}

#[derive(Clone)]
pub struct Buffer<D: BufferDataTrait> {
    handle: Handle<BufferInner>,
    phantom: std::marker::PhantomData<fn() -> D>,
}

pub struct BufferUntyped {
    handle: Handle<BufferInner>,
}

impl<D: BufferDataTrait> Buffer<D> {
    pub fn untyped(&self) -> BufferUntyped {
        BufferUntyped {
            handle: self.handle.clone(),
        }
    }
}

#[derive(Clone)]
pub struct BufferInner {
    buffer_usage: BufferUsage,
    index: u32,
}

impl BufferInner {
    pub fn buffer_usage(&self) -> BufferUsage {
        self.buffer_usage
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BufferUsage {
    Data,
    Index,
}

#[derive(Clone)]
struct UniformBlockInfo {
    size_bytes: u32,
    location: u32,
}

#[derive(Clone)]
struct UniformInfo {
    uniform_type: u32,
    location: u32,
}

#[derive(Clone, Debug)]
struct VertexAttributeInfo {
    byte_size: u32,
    location: u32,
}

#[derive(Clone)]
pub struct VertexAttribute<D: BufferDataTrait> {
    pipeline_index: u32,
    info: Option<VertexAttributeInfo>,
    phantom: std::marker::PhantomData<fn() -> D>,
}

impl<D: BufferDataTrait> VertexAttribute<D> {
    pub fn untyped(&self) -> VertexAttributeUntyped {
        VertexAttributeUntyped {
            // pipeline_index: self.pipeline_index,
            info: self.info.clone(),
        }
    }
}

pub struct VertexAttributeUntyped {
    // pipeline_index: u32,
    info: Option<VertexAttributeInfo>,
}

impl Pipeline {
    pub fn get_vertex_attribute<D: BufferDataTrait>(
        &self,
        name: &str,
    ) -> Result<VertexAttribute<D>, String> {
        if let Some(attribute) = self.0.inner().vertex_attributes.get(name) {
            // TODO: More than just size should be checked, the type should be checked as well.
            if attribute.byte_size == std::mem::size_of::<D>() as u32 {
                Ok(VertexAttribute {
                    pipeline_index: self.0.inner().program_index,
                    info: Some(attribute.clone()),
                    phantom: std::marker::PhantomData,
                })
            } else {
                Err(format!(
                    "Vertex attribute size mismatch for {:?}. /n Shader: {:?}, Rust: {:?}",
                    name,
                    attribute.byte_size,
                    std::mem::size_of::<D>()
                ))
            }
        } else {
            Ok(VertexAttribute {
                pipeline_index: self.0.inner().program_index,
                info: None,
                phantom: std::marker::PhantomData,
            })
        }
    }
}

pub trait GraphicsDataTrait {}

pub trait BufferDataTrait: 'static {}
impl BufferDataTrait for f32 {}
impl BufferDataTrait for u32 {}
impl<const N: usize> BufferDataTrait for [f32; N] {}
impl<const N: usize> BufferDataTrait for [u32; N] {}

pub(crate) unsafe fn slice_to_bytes<T>(t: &[T]) -> &[u8] {
    let ptr = t.as_ptr() as *const u8;
    let size = std::mem::size_of::<T>() * t.len();
    std::slice::from_raw_parts(ptr, size)
}
