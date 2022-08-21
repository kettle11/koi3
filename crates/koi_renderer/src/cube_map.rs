use crate::{Mesh, Renderer, Shader};
use kgraphics::{CommandBufferTrait, GraphicsContextTrait, TextureSettings};
use kgraphics::{PipelineTrait, RenderPassTrait};
use kmath::*;
use koi_assets::{AssetStore, AssetTrait};
use koi_resources::Resources;

pub struct CubeMap(pub(crate) kgraphics::CubeMap);

impl AssetTrait for CubeMap {
    type Settings = ();
}

pub fn initialize_cube_maps(resources: &mut Resources) {
    async fn load(path: String, _settings: ()) -> Option<crate::TextureResult> {
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
        _settings: (),
        resources: &Resources,
    ) -> Option<CubeMap> {
        Some(match source.data {
            crate::TextureData::Bytes(data) => equirectangular_to_cubemap(
                resources,
                data.as_u8_array(),
                source.width,
                source.height,
            ),
            _ => unreachable!(),
        })
    }

    let placeholder =
        equirectangular_to_cubemap(resources, &[0; std::mem::size_of::<f32>() * 4], 1, 1);

    let cube_maps =
        koi_assets::AssetStore::new_with_load_functions(placeholder, load, finalize_load);
    resources.add(cube_maps);
}

pub fn equirectangular_to_cubemap(
    resources: &Resources,
    equirectangular_texture: &[u8],
    width: u32,
    height: u32,
) -> CubeMap {
    // TODO: Convert incoming data into correct color space.
    // TODO: Consider normalizing incoming data to get it in expected range.

    // TODO: This function has a lot of manual memory management and it would be
    // good to figure out how to make this and other low-level-ish graphics code nicer.
    let mut renderer = resources.get::<Renderer>();
    let meshes = resources.get::<AssetStore<Mesh>>();
    let shaders = resources.get::<AssetStore<Shader>>();

    let projection: Mat4 =
        kmath::projection_matrices::perspective_gl(90.0_f32.to_radians(), 1.0, 0.1, 10.);

    // I assume the -Y here is to flip the image as well.
    let views = [
        Mat4::looking_at(Vec3::ZERO, Vec3::X, -Vec3::Y),
        Mat4::looking_at(Vec3::ZERO, -Vec3::X, -Vec3::Y),
        Mat4::looking_at(Vec3::ZERO, Vec3::Y, Vec3::Z),
        Mat4::looking_at(Vec3::ZERO, -Vec3::Y, -Vec3::Z),
        Mat4::looking_at(Vec3::ZERO, Vec3::Z, -Vec3::Y),
        Mat4::looking_at(Vec3::ZERO, -Vec3::Z, -Vec3::Y),
    ];

    let graphics = &mut renderer.raw_graphics_context;

    let equirectangular_texture = graphics
        .new_texture(
            width as _,
            height as _,
            Some(equirectangular_texture),
            kgraphics::PixelFormat::RGBA32F,
            TextureSettings {
                wrapping_horizontal: kgraphics::WrappingMode::ClampToEdge,
                wrapping_vertical: kgraphics::WrappingMode::ClampToEdge,
                minification_filter: kgraphics::FilterMode::Linear,
                magnification_filter: kgraphics::FilterMode::Linear,
                generate_mipmaps: false,
                srgb: false,
                ..Default::default()
            },
        )
        .unwrap();

    let face_size = 512;
    // Hardcode the cube map's size for now.
    let cube_map = graphics
        .new_cube_map(
            face_size,
            face_size,
            None,
            kgraphics::PixelFormat::RGBA16F,
            TextureSettings {
                wrapping_horizontal: kgraphics::WrappingMode::ClampToEdge,
                wrapping_vertical: kgraphics::WrappingMode::ClampToEdge,
                minification_filter: kgraphics::FilterMode::Linear,
                magnification_filter: kgraphics::FilterMode::Linear,
                generate_mipmaps: false,
                srgb: false,
                ..Default::default()
            },
        )
        .unwrap();

    let shader = shaders.get(&Shader::EQUIRECTANGULAR_TO_CUBE_MAP);
    let cube_mesh = meshes.get(&Mesh::CUBE_MAP_CUBE).gpu_mesh.as_ref().unwrap();

    println!("DRAWING CUBE MAP");
    for (i, view) in views.iter().enumerate() {
        let mut raw_command_buffer = graphics.new_command_buffer();
        let face_texture = cube_map.get_face_texture(i);

        let framebuffer = graphics.new_framebuffer(Some(&face_texture), None, None);

        let mut render_pass = raw_command_buffer
            .begin_render_pass_with_framebuffer(&framebuffer, Some((1.0, 1.0, 0.0, 1.0)));

        render_pass.set_uniform_block(&kgraphics::UniformBlock::<()>::from_location(0), None);
        render_pass.set_viewport(0, 0, face_size as u32, face_size as u32);
        render_pass.set_pipeline(&shader.pipeline);
        render_pass.set_mat4_property(
            &shader.pipeline.get_mat4_property("p_projection").unwrap(),
            projection.as_array(),
        );
        render_pass.set_mat4_property(
            &shader.pipeline.get_mat4_property("p_view").unwrap(),
            view.as_array(),
        );

        render_pass.set_texture_property(
            &shader
                .pipeline
                .get_texture_property("p_equirectangular_texture")
                .unwrap(),
            Some(&equirectangular_texture),
            0,
        );
        render_pass.set_vertex_attribute(
            &shader.pipeline.get_vertex_attribute("a_position").unwrap(),
            Some(&cube_mesh.positions),
        );

        render_pass.set_vertex_attribute::<Vec2>(
            &shader
                .pipeline
                .get_vertex_attribute("texture_coordinate_attributes")
                .unwrap(),
            None,
        );
        render_pass.set_vertex_attribute::<Vec3>(
            &shader.pipeline.get_vertex_attribute("a_normal").unwrap(),
            None,
        );

        render_pass.set_vertex_attribute::<Vec4>(
            &shader.pipeline.get_vertex_attribute("a_color").unwrap(),
            None,
        );

        // render_pass.draw_triangles(
        //     cube_mesh.index_end - cube_mesh.index_start,
        //     &cube_mesh.index_buffer,
        // );
        graphics.commit_command_buffer(raw_command_buffer);
        graphics.delete_framebuffer(framebuffer);
    }

    println!("DONE RENDERING CUBE MAP");
    graphics.delete_texture(equirectangular_texture);
    CubeMap(cube_map)
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
        pixel_format: kgraphics::PixelFormat::RGBA32F,
    })
}
