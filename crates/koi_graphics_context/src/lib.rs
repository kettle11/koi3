mod kmath_impls;
pub use kmath_impls::*;

mod assets;
use assets::*;

pub mod backend_trait;

mod bump_allocator;
mod command_buffer;
pub use command_buffer::*;

#[cfg(any(feature = "gl", feature = "webgl"))]
mod gl_shared;

#[cfg(all(target_arch = "wasm32", feature = "webgl"))]
mod webgl_backend;

#[cfg(all(not(target_arch = "wasm32"), feature = "gl"))]
mod gl_backend;

#[cfg(feature = "webgpu")]
mod webgpu_backend;

mod graphics_context;
pub use graphics_context::*;

#[derive(Clone, Copy, Debug)]
pub enum ColorSpace {
    SRGB,
    DisplayP3,
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
pub struct Texture(Handle<TextureInner>);

#[derive(Clone)]
pub struct TextureInner {
    index: u32,
    texture_type: TextureType,
    pixel_format: PixelFormat,
    #[allow(unused)]
    mip: u8,
}

#[derive(Clone)]

pub struct CubeMap(Handle<CubeMapInner>);

#[derive(Clone)]
pub struct CubeMapInner {
    index: u32,
    pixel_format: PixelFormat,
}

#[derive(Debug, Clone)]
pub enum TextureType {
    Texture,
    RenderBuffer,
    CubeMapFace { face: u8 },
    DefaultFramebuffer,
}

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
impl TextureDataTrait for [half::f16; 4] {
    const PIXEL_FORMAT: PixelFormat = PixelFormat::RGBA16F;
}
impl TextureDataTrait for kmath::Vec4 {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    #[allow(unused)]
    pub(crate) uniform_blocks: Vec<UniformBlockInfo>,
    pub(crate) vertex_attributes: std::collections::HashMap<String, VertexAttributeInfo>,
}

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
#[allow(unused)]
struct UniformBlockInfo {
    size_bytes: u32,
    location: u32,
}

#[derive(Debug)]
pub struct Uniform<U: UniformTypeTrait> {
    uniform_info: UniformInfo,
    phantom: std::marker::PhantomData<U>,
}

pub trait UniformTypeTrait: 'static {
    const UNIFORM_TYPE: UniformType;
}

impl UniformTypeTrait for i32 {
    const UNIFORM_TYPE: UniformType = UniformType::Int(1);
}

impl UniformTypeTrait for f32 {
    const UNIFORM_TYPE: UniformType = UniformType::Float(1);
}

impl<const N: usize> UniformTypeTrait for [i32; N] {
    const UNIFORM_TYPE: UniformType = UniformType::Int(N as u8);
}

impl<const N: usize> UniformTypeTrait for [f32; N] {
    const UNIFORM_TYPE: UniformType = UniformType::Float(N as u8);
}

impl UniformTypeTrait for (f32, f32, f32) {
    const UNIFORM_TYPE: UniformType = UniformType::Vec3(1);
}

impl UniformTypeTrait for (f32, f32, f32, f32) {
    const UNIFORM_TYPE: UniformType = UniformType::Vec4(1);
}

// TODO: More uniform types

#[derive(Debug, Clone)]
struct UniformInfo {
    pub(crate) pipeline_index: u32,
    uniform_type: UniformType,
    location: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UniformType {
    UInt(u8),
    Int(u8),
    Float(u8),
    Vec2(u8),
    Vec3(u8),
    Vec4(u8),
    Mat4(u8),
    Sampler2d,
    Sampler3d,
    SamplerCube,
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
    pub fn get_uniform<U: UniformTypeTrait>(&self, name: &str) -> Result<Uniform<U>, String> {
        if let Some(uniform_info) = self.0.inner().uniforms.get(name) {
            if uniform_info.uniform_type != U::UNIFORM_TYPE {
                return Err(format!(
                    "Incorrect uniform type. In shader: {:?}. Passed in: {:?}",
                    uniform_info.uniform_type,
                    U::UNIFORM_TYPE
                ));
            }
            Ok(Uniform {
                uniform_info: uniform_info.clone(),
                phantom: std::marker::PhantomData,
            })
        } else {
            Ok(Uniform {
                uniform_info: UniformInfo {
                    pipeline_index: self.0.inner().program_index,
                    uniform_type: U::UNIFORM_TYPE,
                    location: None,
                },
                phantom: std::marker::PhantomData,
            })
        }
    }

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
impl BufferDataTrait for i32 {}
impl BufferDataTrait for f32 {}
impl BufferDataTrait for u32 {}
impl<const N: usize> BufferDataTrait for [f32; N] {}
impl<const N: usize> BufferDataTrait for [u32; N] {}

pub(crate) unsafe fn slice_to_bytes<T>(t: &[T]) -> &[u8] {
    let ptr = t.as_ptr() as *const u8;
    let size = std::mem::size_of::<T>() * t.len();
    std::slice::from_raw_parts(ptr, size)
}
