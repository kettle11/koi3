use crate::*;

pub(crate) type GLenum = core::ffi::c_uint;

pub const HALF_FLOAT: GLenum = 0x140B;
pub const FLOAT: GLenum = 0x1406;
pub const UNSIGNED_SHORT: GLenum = 0x1403;
pub const UNSIGNED_INT: GLenum = 0x1405;
pub const UNSIGNED_BYTE: GLenum = 0x1401;

pub const DEPTH_COMPONENT16: GLenum = 0x81A5;
pub const DEPTH_COMPONENT24: GLenum = 0x81A6;
pub const DEPTH_COMPONENT32F: GLenum = 0x8CAC;

pub const NEAREST: GLenum = 0x2600;
pub const LINEAR: GLenum = 0x2601;
pub const NEAREST_MIPMAP_NEAREST: GLenum = 0x2700;
pub const LINEAR_MIPMAP_NEAREST: GLenum = 0x2701;
pub const NEAREST_MIPMAP_LINEAR: GLenum = 0x2702;
pub const LINEAR_MIPMAP_LINEAR: GLenum = 0x2703;

pub const CLAMP_TO_EDGE: GLenum = 0x812F;
pub const MIRRORED_REPEAT: GLenum = 0x8370;
pub const REPEAT: GLenum = 0x2901;

pub const DEPTH_COMPONENT: GLenum = 0x1902;
pub const RED: GLenum = 0x1903;
pub const RG: GLenum = 0x8227;
pub const RGB: GLenum = 0x1907;
pub const RGBA: GLenum = 0x1908;

pub const R8: GLenum = 0x8229;
pub const RG8: GLenum = 0x822B;
pub const RGB8: GLenum = 0x8051;
pub const RGBA8: GLenum = 0x8058;
pub const SRGB8_ALPHA8: GLenum = 0x8C43;

pub const RGBA16F: GLenum = 0x881A;
pub const RGBA32F: GLenum = 0x8814;
pub const TEXTURE_2D: GLenum = 0x0DE1;
pub const TEXTURE_CUBE_MAP_POSITIVE_X: GLenum = 0x8515;
pub const TEXTURE_CUBE_MAP: GLenum = 0x8513;

pub const ELEMENT_ARRAY_BUFFER: GLenum = 0x8893;
pub const ARRAY_BUFFER: GLenum = 0x8892;

pub const ONE: GLenum = 1;
pub const ONE_MINUS_SRC_ALPHA: GLenum = 0x0303;
pub const SRC_ALPHA: GLenum = 0x0302;
pub const ALWAYS: GLenum = 0x0207;
pub const LESS: GLenum = 0x0201;
pub const LEQUAL: GLenum = 0x0203;
pub const GREATER: GLenum = 0x0204;
pub const GEQUAL: GLenum = 0x0206;

pub const BACK: GLenum = 0x0405;
pub const FRONT: GLenum = 0x0404;
pub const FRONT_AND_BACK: GLenum = 0x0408;
pub const TEXTURE0: GLenum = 0x84C0;
pub const TEXTURE_3D: GLenum = 0x806F;
pub const ACTIVE_UNIFORMS: GLenum = 0x8B86;

pub const INT: GLenum = 0x1404;
pub const FLOAT_VEC2: GLenum = 0x8B50;
pub const FLOAT_VEC3: GLenum = 0x8B51;
pub const FLOAT_VEC4: GLenum = 0x8B52;
pub const FLOAT_MAT4: GLenum = 0x8B5C;

pub const ACTIVE_ATTRIBUTES: GLenum = 0x8B89;
pub const ACTIVE_UNIFORM_BLOCKS: GLenum = 0x8A36;

pub fn gl_uniform_type_to_uniform_type(gl_enum: GLenum, size_members: u8) -> UniformType {
    match gl_enum {
        FLOAT => UniformType::Float(size_members),
        FLOAT_VEC2 => UniformType::Vec2(size_members),
        FLOAT_VEC3 => UniformType::Vec3(size_members),
        FLOAT_VEC4 => UniformType::Vec4(size_members),
        FLOAT_MAT4 => UniformType::Mat4(size_members),
        INT => UniformType::Int(size_members),
        UNSIGNED_INT => UniformType::UInt(size_members),
        SAMPLER_2D => UniformType::Sampler2d,
        SAMPLER_3D => UniformType::Sampler3d,
        SAMPLER_CUBE => UniformType::SamplerCube,
        _ => {
            panic!("UNIMPLEMENTED UNIFORM TYPE: {:?}", gl_enum);
        }
    }
}
// Useful reference: https://webgl2fundamentals.org/webgl/lessons/webgl-data-textures.html
pub fn pixel_format_to_gl_format_and_inner_format_and_type(
    pixel_format: PixelFormat,
    srgb: bool,
) -> (GLenum, GLenum, GLenum) {
    if srgb {
        assert_eq!(pixel_format, PixelFormat::RGBA8Unorm);
        return (RGBA, SRGB8_ALPHA8, UNSIGNED_BYTE);
    }
    let format = match pixel_format {
        PixelFormat::R8Unorm => RED,
        PixelFormat::RG8Unorm => RG,
        PixelFormat::RGB8Unorm /*| PixelFormat::RGB32F | PixelFormat::RGB16F*/ => RGB,
        PixelFormat::RGBA8Unorm  | PixelFormat::RGBA16F | PixelFormat::RGBA32F => RGBA,
        PixelFormat::Depth16 | PixelFormat::Depth24 | PixelFormat::Depth32F => DEPTH_COMPONENT,
    };

    let inner_format = match pixel_format {
        PixelFormat::Depth16 => DEPTH_COMPONENT16,
        PixelFormat::Depth24 => DEPTH_COMPONENT24,
        PixelFormat::Depth32F => DEPTH_COMPONENT32F,
        PixelFormat::R8Unorm => R8,
        PixelFormat::RG8Unorm => RG8,
        PixelFormat::RGB8Unorm => RGB8,
        PixelFormat::RGBA8Unorm => RGBA8,
        PixelFormat::RGBA16F => RGBA16F,
        PixelFormat::RGBA32F => RGBA32F
        // PixelFormat::RGB16F => RGB16F,
        // PixelFormat::RGB32F => RGB32F,
    };

    let type_ = match pixel_format {
        PixelFormat::Depth16 => UNSIGNED_SHORT,
        PixelFormat::Depth24 => UNSIGNED_INT,
        // PixelFormat::RGB16F => HALF_FLOAT,
        PixelFormat::RGBA16F => HALF_FLOAT,
        PixelFormat::Depth32F | PixelFormat::RGBA32F => FLOAT,
        _ => UNSIGNED_BYTE,
    };

    (format, inner_format, type_)
}

pub fn minification_filter_to_gl_enum(
    minification_filter_mode: FilterMode,
    mipmap_filter_mode: FilterMode,
    has_mipmaps: bool,
) -> GLenum {
    if has_mipmaps {
        match (minification_filter_mode, mipmap_filter_mode) {
            (FilterMode::Nearest, FilterMode::Nearest) => NEAREST_MIPMAP_NEAREST,
            (FilterMode::Nearest, FilterMode::Linear) => NEAREST_MIPMAP_LINEAR,
            (FilterMode::Linear, FilterMode::Nearest) => LINEAR_MIPMAP_NEAREST,
            (FilterMode::Linear, FilterMode::Linear) => LINEAR_MIPMAP_LINEAR,
        }
    } else {
        match minification_filter_mode {
            FilterMode::Nearest => NEAREST,
            FilterMode::Linear => LINEAR,
        }
    }
}

pub fn magnification_filter_to_gl_enum(filter_mode: FilterMode) -> GLenum {
    match filter_mode {
        FilterMode::Nearest => NEAREST,
        FilterMode::Linear => LINEAR,
    }
}

pub fn wrapping_to_gl_enum(wrapping_mode: WrappingMode) -> GLenum {
    match wrapping_mode {
        WrappingMode::ClampToEdge => CLAMP_TO_EDGE,
        WrappingMode::MirrorRepeat => MIRRORED_REPEAT,
        WrappingMode::Repeat => REPEAT,
    }
}
