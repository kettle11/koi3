use crate::{spherical_harmonics::SphericalHarmonics, Renderer};
use kmath::*;
use koi_assets::AssetTrait;
use koi_resources::Resources;

pub struct CubeMap {
    pub(crate) texture: koi_graphics_context::CubeMap,
    /// Used for efficient irradiance.
    pub spherical_harmonics: SphericalHarmonics<4>,
    pub brightest_direction: Vec3,
}

#[derive(Clone)]
pub struct CubeMapSettings {
    pub luminance_of_brightest_pixel: Option<f32>,
}

pub mod luminance {
    pub const SUNRISE_OR_SUNSET_PHOTO: f32 = 25.0;
    pub const CLOUDY_DAY: f32 = 2_000.0;
    pub const TYPICAL_SUNLIT_SCENE: f32 = 5_000.0;
    pub const CLOUD: f32 = 5_000.0;
    pub const CLEAR_SKY: f32 = 7_000.0;
    pub const SUN_AT_NOON: f32 = 1000_000_000.0;
}

impl Default for CubeMapSettings {
    fn default() -> Self {
        Self {
            /// Candelas per meter squared
            /// ttps://en.wikipedia.org/wiki/Orders_of_magnitude_(luminance)
            luminance_of_brightest_pixel: None,
        }
    }
}

impl AssetTrait for CubeMap {
    type Settings = CubeMapSettings;
}

pub fn initialize_cube_maps(resources: &mut Resources) {
    async fn load(path: String, _settings: CubeMapSettings) -> Option<crate::TextureResult> {
        let extension = std::path::Path::new(&path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .expect("Expected cube map file extension")
            .to_lowercase();

        match &*extension {
            #[cfg(feature = "hdri")]
            "hdr" => {
                let bytes = koi_fetch::fetch_bytes(&path)
                    .await
                    .unwrap_or_else(|_| panic!("Failed to open file: {}", path));
                Some(hdri_data_from_bytes(&bytes)?)
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
        source: crate::TextureResult,
        settings: CubeMapSettings,
        resources: &Resources,
    ) -> Option<CubeMap> {
        Some(match source.data {
            crate::TextureData::Bytes(data) => prepare_cubemap(
                resources,
                data.as_u8_array(),
                source.width,
                source.height,
                settings,
            ),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        })
    }

    let placeholder = prepare_cubemap(
        resources,
        &[0; std::mem::size_of::<f32>() * 4],
        1,
        1,
        CubeMapSettings::default(),
    );

    let cube_maps =
        koi_assets::AssetStore::new_with_load_functions(placeholder, load, finalize_load);
    resources.add(cube_maps);
}

pub fn prepare_cubemap(
    resources: &Resources,
    equirectangular_texture: &[u8],
    equirectangular_width: u32,
    equirectangular_height: u32,
    settings: CubeMapSettings,
) -> CubeMap {
    // TODO: Convert incoming data into correct color space.
    // TODO: Consider normalizing incoming data to get it in expected range.

    let mut renderer = resources.get::<Renderer>();

    let face_size = 512
        .min(equirectangular_width as usize)
        .min(equirectangular_height as usize);
    let graphics = &mut renderer.raw_graphics_context;

    assert_eq!(
        equirectangular_texture.len(),
        (equirectangular_width * equirectangular_height) as usize * std::mem::size_of::<Vec4>()
    );
    let equirectangular_data: &[Vec4] = unsafe { std::mem::transmute(equirectangular_texture) };

    // TODO: Do this off the main thread.
    let mut output_faces0 = equirectangular_to_cubemap_cpu(
        equirectangular_data,
        equirectangular_width as _,
        equirectangular_height as _,
        face_size,
    );

    let output_faces = [
        output_faces0[0].as_slice(),
        output_faces0[1].as_slice(),
        output_faces0[2].as_slice(),
        output_faces0[3].as_slice(),
        output_faces0[4].as_slice(),
        output_faces0[5].as_slice(),
    ];

    let (brightest_value, brightest_direction) = get_brightest_value_and_direction(&output_faces);
    println!("BRIGHTEST VALUE: {:?}", brightest_value);

    let mut spherical_harmonics = SphericalHarmonics::from_cube_map(&output_faces);

    println!(
        "LUMINANCE OF BRIGHTEST PIXEL: {:?}",
        settings.luminance_of_brightest_pixel
    );
    if let Some(luminance_of_brightest_pixel) = settings.luminance_of_brightest_pixel {
        let scale_factor = luminance_of_brightest_pixel / brightest_value;
        println!("SCALE FACTOR: {:?}", scale_factor);
        for face in output_faces0.iter_mut() {
            for d in face.iter_mut() {
                *d = (d.xyz() * scale_factor).extend(d.w);
            }
        }
        spherical_harmonics.scale(scale_factor);
    }

    let output_faces = [
        output_faces0[0].as_slice(),
        output_faces0[1].as_slice(),
        output_faces0[2].as_slice(),
        output_faces0[3].as_slice(),
        output_faces0[4].as_slice(),
        output_faces0[5].as_slice(),
    ];

    let settings = koi_graphics_context::TextureSettings {
        wrapping_horizontal: koi_graphics_context::WrappingMode::ClampToEdge,
        wrapping_vertical: koi_graphics_context::WrappingMode::ClampToEdge,
        minification_filter: koi_graphics_context::FilterMode::Linear,
        magnification_filter: koi_graphics_context::FilterMode::Linear,
        generate_mipmaps: false,
        srgb: false,
        ..Default::default()
    };
    let cube_map = graphics.new_cube_map::<kmath::Vec4>(
        face_size as _,
        face_size as _,
        //Some(output_faces),
        settings,
    );
    // TODO: Make this use f16 instead.
    graphics.update_cube_map::<kmath::Vec4>(&cube_map, &output_faces, settings);

    CubeMap {
        texture: cube_map,
        brightest_direction,
        spherical_harmonics,
    }
}

#[cfg(feature = "hdri")]
fn hdri_data_from_bytes(bytes: &[u8]) -> Option<crate::TextureResult> {
    // This data is always assumed to be linear sRGB

    let image = hdrldr::load(bytes).ok()?;

    // Pad with alpha.
    // Some platforms (Firefox on web) don't support RGB32F well.
    let mut texture: Vec<[f32; 4]> = Vec::with_capacity(image.data.len());
    for hdrldr::RGB { r, g, b } in image.data {
        texture.push([r, g, b, 0.0]);
    }

    Some(crate::TextureResult {
        data: crate::TextureData::Bytes(Box::new(texture)),
        width: image.width as u32,
        height: image.height as u32,
        pixel_format: koi_graphics_context::PixelFormat::RGBA32F,
    })
}

pub fn get_brightest_value_and_direction(data: &[&[Vec4]]) -> (f32, Vec3) {
    let mut brightest_value = f32::MIN;
    let mut brightest_direction = Vec3::ZERO;

    let dim = (data.len() as f32).sqrt() as usize;
    for (face_index, data_out) in data.iter().enumerate() {
        for (i, pixel) in data_out.iter().enumerate() {
            let x = i % dim;
            let y = i / dim;

            let magnitude = pixel.xyz().length_squared();
            if magnitude > brightest_value {
                brightest_value = magnitude;

                // Get a direction for this cube map pixel
                let direction = get_direction_for(face_index, x as f32, y as f32, dim as f32);

                brightest_direction = direction;
            }
        }
    }

    (brightest_value.sqrt(), brightest_direction)
}
pub fn equirectangular_to_cubemap_cpu(
    equirectangular_data: &[Vec4],
    equirectangular_width: usize,
    equirectangular_height: usize,
    output_dim: usize,
) -> [Vec<Vec4>; 6] {
    let equirectangular_width_f32 = equirectangular_width as f32;
    let equirectangular_height_f32 = equirectangular_height as f32;

    const F_1_PI: f32 = 0.318309886183790671537767526745028724;

    /*


        auto toRectilinear = [width, height](float3 s) -> float2 {
            float xf = std::atan2(s.x, s.z) * F_1_PI;   // range [-1.0, 1.0]
            float yf = std::asin(s.y) * (2 * F_1_PI);   // range [-1.0, 1.0]
            xf = (xf + 1.0f) * 0.5f * (width  - 1);        // range [0, width [
            yf = (1.0f - yf) * 0.5f * (height - 1);        // range [0, height[
            return float2(xf, yf);
        };
    */
    let mut data_out = [
        vec![Vec4::ZERO; output_dim * output_dim],
        vec![Vec4::ZERO; output_dim * output_dim],
        vec![Vec4::ZERO; output_dim * output_dim],
        vec![Vec4::ZERO; output_dim * output_dim],
        vec![Vec4::ZERO; output_dim * output_dim],
        vec![Vec4::ZERO; output_dim * output_dim],
    ];

    fn to_rectilinear(width: f32, height: f32, s: Vec3) -> Vec2 {
        let xf = s.x.atan2(s.z) * F_1_PI; // range [-1.0, 1.0]
        let yf = s.y.asin() * (2.0 * F_1_PI); // range [-1.0, 1.0]
        let xf = (xf + 1.0) * 0.5 * (width - 1.0); // range [0, width]
        let yf = (1.0 - yf) * 0.5 * (height - 1.0); // range [0, height]
        return Vec2::new(xf, yf);
    }

    for (face_index, data_out) in data_out.iter_mut().enumerate() {
        for (i, pixel) in data_out.iter_mut().enumerate() {
            let x = i % output_dim;
            let y = i / output_dim;

            // Get a direction for this cube map pixel
            let direction = get_direction_for(face_index, x as f32, y as f32, output_dim as f32);

            // Convert to sample in equirectangular
            let sample_pos = to_rectilinear(
                equirectangular_width_f32,
                equirectangular_height_f32,
                direction,
            );

            let sample = equirectangular_data
                [sample_pos.y as usize * equirectangular_width + sample_pos.x as usize];

            // TODO: Take multiple samples
            *pixel = sample;
        }
    }
    data_out
}

pub(crate) fn get_direction_for(index: usize, x: f32, y: f32, dimensions: f32) -> kmath::Vec3 {
    let x = x + 0.5;
    let y = y + 0.5;
    let m_scale = 2.0 / dimensions;
    // map [0, dim] to [-1,1] with (-1,-1) at bottom left
    let cx: f32 = (x as f32 * m_scale) - 1.0;
    let cy: f32 = 1.0 - (y as f32 * m_scale);

    let l: f32 = (cx * cx + cy * cy + 1.0).sqrt();
    let dir = match index {
        0 => kmath::Vec3::new(1.0, cy, -cx),
        1 => kmath::Vec3::new(-1.0, cy, cx),
        2 => kmath::Vec3::new(cx, 1.0, -cy),
        3 => kmath::Vec3::new(cx, -1.0, cy),
        4 => kmath::Vec3::new(cx, cy, 1.0),
        5 => kmath::Vec3::new(-cx, cy, -1.0),
        _ => unreachable!(),
    };
    dir * (1.0 / l)
}

unsafe fn slice_to_bytes<T>(t: &[T]) -> &[u8] {
    let ptr = t.as_ptr() as *const u8;
    let size = std::mem::size_of::<T>() * t.len();
    std::slice::from_raw_parts(ptr, size)
}
