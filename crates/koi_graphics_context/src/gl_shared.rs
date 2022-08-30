use crate::*;
use core::ffi::c_uint;

pub const HALF_FLOAT: c_uint = 0x140B;
pub const FLOAT: c_uint = 0x1406;
pub const UNSIGNED_SHORT: c_uint = 0x1403;
pub const UNSIGNED_INT: c_uint = 0x1405;
pub const UNSIGNED_BYTE: c_uint = 0x1401;

pub const DEPTH_COMPONENT16: c_uint = 0x81A5;
pub const DEPTH_COMPONENT24: c_uint = 0x81A6;
pub const DEPTH_COMPONENT32F: c_uint = 0x8CAC;

pub const NEAREST: c_uint = 0x2600;
pub const LINEAR: c_uint = 0x2601;
pub const NEAREST_MIPMAP_NEAREST: c_uint = 0x2700;
pub const LINEAR_MIPMAP_NEAREST: c_uint = 0x2701;
pub const NEAREST_MIPMAP_LINEAR: c_uint = 0x2702;
pub const LINEAR_MIPMAP_LINEAR: c_uint = 0x2703;

pub const CLAMP_TO_EDGE: c_uint = 0x812F;
pub const MIRRORED_REPEAT: c_uint = 0x8370;
pub const REPEAT: c_uint = 0x2901;

pub const DEPTH_COMPONENT: c_uint = 0x1902;
pub const RED: c_uint = 0x1903;
pub const RG: c_uint = 0x8227;
pub const RGB: c_uint = 0x1907;
pub const RGBA: c_uint = 0x1908;

pub const R8: c_uint = 0x8229;
pub const RG8: c_uint = 0x822B;
pub const RGB8: c_uint = 0x8051;
pub const RGBA8: c_uint = 0x8058;
pub const SRGB8_ALPHA8: c_uint = 0x8C43;

pub const RGBA16F: c_uint = 0x881A;
pub const RGBA32F: c_uint = 0x8814;

// Useful reference: https://webgl2fundamentals.org/webgl/lessons/webgl-data-textures.html
pub fn pixel_format_to_gl_format_and_inner_format_and_type(
    pixel_format: PixelFormat,
    srgb: bool,
) -> (c_uint, c_uint, c_uint) {
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
) -> c_uint {
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

pub fn magnification_filter_to_gl_enum(filter_mode: FilterMode) -> c_uint {
    match filter_mode {
        FilterMode::Nearest => NEAREST,
        FilterMode::Linear => LINEAR,
    }
}

pub fn wrapping_to_gl_enum(wrapping_mode: WrappingMode) -> c_uint {
    match wrapping_mode {
        WrappingMode::ClampToEdge => CLAMP_TO_EDGE,
        WrappingMode::MirrorRepeat => MIRRORED_REPEAT,
        WrappingMode::Repeat => REPEAT,
    }
}
