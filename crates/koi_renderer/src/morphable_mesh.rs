use crate::*;
use kgraphics::GraphicsContextTrait;
use koi_assets::*;
use koi_resources::Resources;

/*
pub struct MeshMorph {
    pub morphable_mesh_data: Handle<MorphableMeshData>,
    pub(crate) weights: Vec<f32>,
}

impl MeshMorph {
    pub fn new(morphable_mesh_data: Handle<MorphableMeshData>, weights: &[f32]) -> Self {
        Self {
            morphable_mesh_data,
            weights: weights.into(),
        }
    }
    pub fn set_weights(&mut self, weights: &[f32]) {
        self.weights.clear();
        self.weights.extend_from_slice(weights);
    }
*/

pub struct MorphableMeshData {
    pub mesh: Handle<Mesh>,
    pub morph_targets_texture: Handle<Texture>,
}

impl AssetTrait for MorphableMeshData {
    type Settings = ();
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

            let i0 = mesh_data.positions.iter().zip(mesh_data.normals.iter());
            let i1 = morph_target
                .positions
                .iter()
                .zip(morph_target.normals.iter());
            for ((p0, n0), (p1, n1)) in i0.zip(i1) {
                let delta_p = *p1 - *p0;
                let delta_n = *n1 - *n0;
                data.push([
                    half::f16::from_f32(delta_p.x),
                    half::f16::from_f32(delta_p.y),
                    half::f16::from_f32(delta_p.z),
                    half::f16::from_f32(0.0),
                ]);
                data.push([
                    half::f16::from_f32(delta_n.x),
                    half::f16::from_f32(delta_n.y),
                    half::f16::from_f32(delta_n.z),
                    half::f16::from_f32(0.0),
                ]);
            }
        }

        println!("HERE HERE HERE");
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
                       
                        generate_mipmaps: false,
                        ..Default::default()
                    },
                )
                .unwrap(),
        ));
        println!("HERE HERE HERE: {:?}", dimension_needed);

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
