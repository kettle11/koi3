use crate::Texture;
use kgraphics::GraphicsContextTrait;
use koi_assets::*;
use koi_resources::Resources;

impl Texture {
    pub const WHITE: Handle<Texture> = Handle::<Texture>::from_index(0);
    pub const BLACK: Handle<Texture> = Handle::<Texture>::from_index(1);

    /// A texture that produces normals that all face outwards.
    /// The color is (0.5, 0.5, 1.0)
    pub const DEFAULT_NORMAL: Handle<Texture> = Handle::<Texture>::from_index(2);
}

impl AssetTrait for Texture {
    type Settings = kgraphics::TextureSettings;
}

pub struct TextureResult {
    width: u32,
    height: u32,
    pixel_format: kgraphics::PixelFormat,
    data: TextureData,
}

pub fn initialize_textures(renderer: &mut crate::Renderer) -> koi_assets::AssetStore<Texture> {
    let texture = Texture(
        renderer
            .raw_graphics_context
            .new_texture(
                1,
                1,
                Some(&[255, 255, 255, 255]),
                kgraphics::PixelFormat::RGBA8Unorm,
                kgraphics::TextureSettings {
                    srgb: false,
                    ..Default::default()
                },
            )
            .unwrap(),
    );

    async fn load(path: String, settings: kgraphics::TextureSettings) -> Option<TextureResult> {
        let extension = std::path::Path::new(&path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .expect("Expected image file extension")
            .to_lowercase();

        match &*extension {
            #[cfg(feature = "png")]
            "png" => {
                let bytes = match std::fs::read(&path) {
                    Ok(b) => b,
                    Err(_) => {
                        println!("Could not open path: {:?}", path);
                        None?
                    }
                };
                let imagine::ImageRGBA8 {
                    width,
                    height,
                    mut pixels,
                } = imagine::ImageRGBA8::try_from_png_bytes(&bytes).unwrap();

                // TODO: Need to convert to appropriate color space here (if necessary)

                // Premultiply texture
                for v in pixels.iter_mut() {
                    let a = v[3] as f32 / 255.0;
                    v[0] = (v[0] as f32 * a) as u8;
                    v[1] = (v[1] as f32 * a) as u8;
                    v[2] = (v[2] as f32 * a) as u8;
                }

                Some(TextureResult {
                    data: TextureData::Bytes(Box::new(pixels)),
                    pixel_format: kgraphics::PixelFormat::RGBA8Unorm,
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

                let bytes = match std::fs::read(&path) {
                    Ok(b) => b,
                    Err(_) => {
                        println!("Could not open path: {:?}", path);
                        None?
                    }
                };
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
                            kgraphics::PixelFormat::RGBA8Unorm
                        } else {
                            kgraphics::PixelFormat::RGB8Unorm
                        }
                    }
                    jpeg_decoder::PixelFormat::L8 => {
                        // Convert to RGBA sRGB8_ALPHA8 is the only color renderable format
                        // which is required for mipmap generation
                        if settings.srgb {
                            pixels = extend_pixels_1_with_alpha(pixels);
                            kgraphics::PixelFormat::RGBA8Unorm
                        } else {
                            kgraphics::PixelFormat::R8Unorm
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
                println!(
                    "Error loading image. Unsupported file extension: {extension} for path {path}"
                );
                return None;
            }
        }
    }

    fn finalize_load(
        source: TextureResult,
        settings: kgraphics::TextureSettings,
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
        Texture(
            renderer
                .raw_graphics_context
                .new_texture(
                    1,
                    1,
                    Some(&[0, 0, 0, 255]),
                    kgraphics::PixelFormat::RGBA8Unorm,
                    kgraphics::TextureSettings {
                        srgb: false,
                        ..Default::default()
                    },
                )
                .unwrap(),
        ),
        &Texture::BLACK,
    );

    textures.add_and_leak(
        Texture(
            renderer
                .raw_graphics_context
                .new_texture(
                    1,
                    1,
                    Some(&[128, 128, 255, 255]),
                    kgraphics::PixelFormat::RGBA8Unorm,
                    kgraphics::TextureSettings {
                        srgb: false,
                        ..Default::default()
                    },
                )
                .unwrap(),
        ),
        &Texture::DEFAULT_NORMAL,
    );
    textures
}

pub fn new_texture_from_texture_load_data(
    graphics: &mut kgraphics::GraphicsContext,
    texture_load_data: TextureResult,
    texture_settings: kgraphics::TextureSettings,
) -> Texture {
    Texture(match texture_load_data.data {
        TextureData::Bytes(data) => graphics
            .new_texture(
                texture_load_data.width,
                texture_load_data.height,
                Some(data.as_u8_array()),
                texture_load_data.pixel_format,
                texture_settings,
            )
            .unwrap(),
        #[cfg(target_arch = "wasm32")]
        TextureData::JSObject(data) => Texture(
            graphics
                .context
                .new_texture_from_js_object(
                    texture_load_data.width,
                    texture_load_data.height,
                    &data,
                    texture_load_data.pixel_format,
                    texture_settings,
                )
                .unwrap(),
        ),
    })
}

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

impl<T: bytemuck::Pod + Send + Sync + 'static> AsU8Array for Vec<T> {
    fn as_u8_array(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
    fn as_u8_array_mut(&mut self) -> &mut [u8] {
        bytemuck::cast_slice_mut(self)
    }
}
