use crate::Texture;
use koi_assets::*;
use koi_graphics_context::TextureSettings;
use koi_resources::Resources;

impl Texture {
    pub const WHITE: Handle<Texture> = Handle::<Texture>::from_index(0);
    pub const BLACK: Handle<Texture> = Handle::<Texture>::from_index(1);

    /// A texture that produces normals that all face outwards.
    /// The color is (0.5, 0.5, 1.0)
    pub const DEFAULT_NORMAL: Handle<Texture> = Handle::<Texture>::from_index(2);
}

impl AssetTrait for Texture {
    type Settings = koi_graphics_context::TextureSettings;
}

pub(crate) struct TextureResult {
    pub width: u32,
    pub height: u32,
    pub pixel_format: koi_graphics_context::PixelFormat,
    pub data: TextureData,
}

pub fn new_texture_from_bytes(
    graphics: &mut koi_graphics_context::GraphicsContext,
    extension: &str,
    bytes: &[u8],
    settings: TextureSettings,
) -> Option<Texture> {
    let texture_result = texture_result_from_extension_and_bytes(extension, bytes, settings)?;
    Some(new_texture_from_texture_load_data(
        graphics,
        texture_result,
        settings,
    ))
}

#[allow(unused)]
pub(crate) fn texture_result_from_extension_and_bytes(
    extension: &str,
    bytes: &[u8],
    settings: TextureSettings,
) -> Option<TextureResult> {
    match &*extension {
        #[cfg(feature = "png")]
        "png" => {
            let imagine::Bitmap::<pixel_formats::r8g8b8a8_Srgb> {
                width,
                height,
                pixels,
            } = imagine::png::png_try_bitmap_rgba(&bytes, true).unwrap();

            // TODO: Need to convert to appropriate color space here (if necessary)

            // Premultiply texture
            /*
            for v in pixels.iter_mut() {
                let a = v.a as f32 / 255.0;
                v.r = (v.r as f32 * a) as u8;
                v.g = (v.g as f32 * a) as u8;
                v.b = (v.b as f32 * a) as u8;
            }
            */

            Some(TextureResult {
                data: TextureData::Bytes(Box::new(pixels)),
                pixel_format: koi_graphics_context::PixelFormat::RGBA8Unorm,
                width: width as _,
                height: height as _,
            })
        }
        #[cfg(feature = "jpeg")]
        "jpg" | "jpeg" => {
            #[allow(unused)]
            fn extend_pixels_1_with_alpha(pixels: Vec<u8>) -> Vec<u8> {
                pixels.iter().flat_map(|a| [*a, *a, *a, 255]).collect()
            }

            #[allow(unused)]
            fn extend_pixels_3_with_alpha(pixels: Vec<u8>) -> Vec<u8> {
                pixels
                    .chunks_exact(3)
                    .flat_map(|a| [a[0], a[1], a[2], 255])
                    .collect()
            }

            let reader = std::io::BufReader::new(&*bytes);

            let mut decoder = jpeg_decoder::Decoder::new(reader);
            let mut pixels = decoder.decode().expect("failed to decode image");
            let metadata = decoder.info().unwrap();

            let pixel_format = match metadata.pixel_format {
                jpeg_decoder::PixelFormat::RGB24 => {
                    // Convert to RGBA sRGB8_ALPHA8 is the only color renderable format
                    // which is required for mipmap generation
                    if settings.srgb {
                        pixels = extend_pixels_3_with_alpha(pixels);
                        koi_graphics_context::PixelFormat::RGBA8Unorm
                    } else {
                        koi_graphics_context::PixelFormat::RGB8Unorm
                    }
                }
                jpeg_decoder::PixelFormat::L8 => {
                    // Convert to RGBA sRGB8_ALPHA8 is the only color renderable format
                    // which is required for mipmap generation
                    if settings.srgb {
                        pixels = extend_pixels_1_with_alpha(pixels);
                        koi_graphics_context::PixelFormat::RGBA8Unorm
                    } else {
                        koi_graphics_context::PixelFormat::R8Unorm
                    }
                }
                jpeg_decoder::PixelFormat::CMYK32 => {
                    panic!("CMYK is currently unsupported")
                } // _ => unimplemented!("Unsupported Jpeg pixel format: {:?}", metadata.pixel_format,),
            };
            Some(TextureResult {
                data: TextureData::Bytes(Box::new(pixels)),
                pixel_format,
                width: metadata.width as u32,
                height: metadata.height as u32,
            })
        }
        _ => {
            println!("Error loading image. Unsupported image file extension: {extension} ");
            None
        }
    }
}

pub fn initialize_textures(renderer: &mut crate::Renderer) -> koi_assets::AssetStore<Texture> {
    let texture = Texture(renderer.raw_graphics_context.new_texture_with_data(
        1,
        1,
        1,
        &[[255, 255, 255, 255]],
        koi_graphics_context::TextureSettings {
            srgb: false,
            ..Default::default()
        },
    ));

    async fn load(
        path: String,
        #[allow(unused)] settings: koi_graphics_context::TextureSettings,
    ) -> Option<TextureResult> {
        #[allow(unused)]
        let extension = std::path::Path::new(&path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .expect("Expected image file extension")
            .to_lowercase();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let bytes = koi_fetch::fetch_bytes(&path)
                .await
                .unwrap_or_else(|_| panic!("Failed to open file: {}", path));
            texture_result_from_extension_and_bytes(&extension, &bytes, settings)
        }

        // Web uses the browser-native decoders as much faster path.
        // Note: A flag is set in `webgl_backend.js` in `koi_graphics_context` that enables
        // premultiplied textures.
        #[cfg(target_arch = "wasm32")]
        {
            let kwasm::libraries::ImageLoadResult {
                image_js_object,
                width,
                height,
            } = kwasm::libraries::load_image(&path)
                .await
                .unwrap_or_else(|_| panic!("Failed to open file: {}", path));
            Some(TextureResult {
                data: TextureData::JSObject(image_js_object.to_dynamic()),
                width,
                height,
                pixel_format: koi_graphics_context::PixelFormat::RGBA8Unorm,
            })
        }
    }

    fn finalize_load(
        source: TextureResult,
        settings: koi_graphics_context::TextureSettings,
        resources: &Resources,
    ) -> Option<Texture> {
        Some(new_texture_from_texture_load_data(
            &mut resources.get::<crate::Renderer>().raw_graphics_context,
            source,
            settings,
        ))
    }

    let mut textures =
        koi_assets::AssetStore::new_with_load_functions(texture, load, finalize_load);

    textures.add_and_leak(
        Texture(renderer.raw_graphics_context.new_texture_with_data(
            1,
            1,
            1,
            &[[0, 0, 0, 255]],
            koi_graphics_context::TextureSettings {
                srgb: false,
                ..Default::default()
            },
        )),
        &Texture::BLACK,
    );

    textures.add_and_leak(
        Texture(renderer.raw_graphics_context.new_texture_with_data(
            1,
            1,
            1,
            &[[128, 128, 255, 255]],
            koi_graphics_context::TextureSettings {
                srgb: false,
                ..Default::default()
            },
        )),
        &Texture::DEFAULT_NORMAL,
    );
    textures
}

fn new_texture_from_texture_load_data(
    graphics: &mut koi_graphics_context::GraphicsContext,
    texture_load_data: TextureResult,
    texture_settings: koi_graphics_context::TextureSettings,
) -> Texture {
    Texture(match texture_load_data.data {
        TextureData::Bytes(data) => unsafe {
            graphics.new_texture_with_bytes(
                texture_load_data.width,
                texture_load_data.height,
                1,
                data.as_u8_array(),
                texture_load_data.pixel_format,
                texture_settings,
            )
        },
        #[cfg(target_arch = "wasm32")]
        TextureData::JSObject(data) => graphics.new_texture_from_js_object(
            texture_load_data.width,
            texture_load_data.height,
            &data,
            texture_load_data.pixel_format,
            texture_settings,
        ),
    })
}

// Todo: This shouldn't be necessary.
// kwasm::JSObjectDynamic should instead be wrapped in a `NotSyncSend`, but `NotSyncSend` isn't its own crate yet.
unsafe impl Send for TextureData {}
unsafe impl Sync for TextureData {}

pub enum TextureData {
    Bytes(Box<dyn AsU8Array>),
    #[cfg(target_arch = "wasm32")]
    JSObject(kwasm::JSObjectDynamic),
}

/// Used in texture loading to upload data to the GPU.
pub trait AsU8Array: 'static + Send + Sync {
    fn as_u8_array(&self) -> &[u8];
    fn as_u8_array_mut(&mut self) -> &mut [u8];
}

#[cfg(feature = "bytemuck")]
impl<T: bytemuck::Pod + Send + Sync + 'static> AsU8Array for Vec<T> {
    fn as_u8_array(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
    fn as_u8_array_mut(&mut self) -> &mut [u8] {
        bytemuck::cast_slice_mut(self)
    }
}
