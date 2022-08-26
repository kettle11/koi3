use crate::*;
use kgraphics::GraphicsContextTrait;
use koi_assets::*;
use koi_resources::Resources;

pub struct MorphableMesh {}
pub struct MorphableMeshData {
    pub mesh: Handle<Mesh>,
    pub morph_targets_texture: Handle<Texture>,
}

impl MorphableMeshData {
    pub fn new(resources: &Resources, mesh_data: MeshData, morph_targets: &[MeshData]) -> Self {
        let graphics = &mut resources.get::<Renderer>().raw_graphics_context;
        let mut meshes = resources.get::<AssetStore<Mesh>>();
        let mut textures = resources.get::<AssetStore<Texture>>();

        let dimension_needed = ((mesh_data.positions.len() + mesh_data.normals.len()) as f32)
            .sqrt()
            .ceil() as usize;
        let mut data = Vec::with_capacity(dimension_needed * dimension_needed * 4);

        for morph_target in morph_targets {
            assert_eq!(mesh_data.positions.len(), morph_target.positions.len());
            assert_eq!(mesh_data.normals.len(), morph_target.normals.len());

            for (position, normal) in mesh_data.positions.iter().zip(mesh_data.normals.iter()) {
                data.push([
                    half::f16::from_f32(position.x),
                    half::f16::from_f32(position.y),
                    half::f16::from_f32(position.z),
                    half::f16::from_f32(0.0),
                ]);
                data.push([
                    half::f16::from_f32(normal.x),
                    half::f16::from_f32(normal.y),
                    half::f16::from_f32(normal.z),
                    half::f16::from_f32(0.0),
                ]);
            }
        }

        let morph_targets_texture = textures.add(Texture(
            graphics
                .new_texture(
                    dimension_needed as u32,
                    dimension_needed as u32,
                    morph_targets.len() as u32,
                    Some(unsafe { slice_to_bytes(&data) }),
                    kgraphics::PixelFormat::RGBA16F,
                    kgraphics::TextureSettings {
                        srgb: false,
                        minification_filter: kgraphics::FilterMode::Nearest,
                        magnification_filter: kgraphics::FilterMode::Nearest,
                        mipmap_filter: kgraphics::FilterMode::Nearest,
                        generate_mipmaps: false,
                        ..Default::default()
                    },
                )
                .unwrap(),
        ));

        let mesh = meshes.add(Mesh::new(graphics, mesh_data));

        MorphableMeshData {
            mesh,
            morph_targets_texture,
        }
    }
}

unsafe fn slice_to_bytes<T>(t: &[T]) -> &[u8] {
    let ptr = t.as_ptr() as *const u8;
    let size = std::mem::size_of::<T>() * t.len();
    std::slice::from_raw_parts(ptr, size)
}
