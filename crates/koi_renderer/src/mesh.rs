use kmath::*;

pub struct Mesh {
    pub(crate) gpu_mesh: Option<GPUMesh>,
    pub mesh_data: Option<MeshData>,
    pub bounding_box: Option<Box3>,
}

impl Mesh {
    pub fn new(graphics: &mut koi_graphics_context::GraphicsContext, mesh_data: MeshData) -> Self {
        let gpu_mesh = new_gpu_mesh(graphics, &mesh_data).unwrap();
        let bounding_box = Box3::from_points(mesh_data.positions.iter().copied());
        Mesh {
            gpu_mesh: Some(gpu_mesh),
            mesh_data: Some(mesh_data),
            bounding_box: Some(bounding_box),
        }
    }

    pub fn recalculate_bounding_box(&mut self) {
        self.bounding_box = self
            .mesh_data
            .as_ref()
            .map(|mesh_data| Box3::from_points(mesh_data.positions.iter().copied()));
    }

    pub fn update_mesh_on_gpu(&mut self, graphics: &mut koi_graphics_context::GraphicsContext) {
        if let Some(mesh_data) = self.mesh_data.as_ref() {
            self.gpu_mesh = Some(new_gpu_mesh(graphics, mesh_data).unwrap())
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MeshData {
    pub positions: Vec<Vec3>,
    pub indices: Vec<[u32; 3]>,
    pub normals: Vec<Vec3>,
    pub texture_coordinates: Vec<Vec2>,
    /// Colors are linear sRGB
    pub colors: Vec<Vec4>,
}

impl MeshData {
    pub fn apply_transform(&mut self, transform: koi_transform::Transform) {
        let transform_matrix = transform.local_to_world();
        for position in self.positions.iter_mut() {
            *position = transform_matrix.transform_point(*position);
        }

        for normal in self.normals.iter_mut() {
            *normal = transform_matrix.transform_vector(*normal);
        }
    }
}

#[derive(Clone)]
pub struct GPUMesh {
    pub positions: koi_graphics_context::Buffer<kmath::Vec3>,
    pub texture_coordinates: Option<koi_graphics_context::Buffer<kmath::Vec2>>,
    pub normals: Option<koi_graphics_context::Buffer<kmath::Vec3>>,
    pub colors: Option<koi_graphics_context::Buffer<kmath::Vec4>>,
    pub index_buffer: koi_graphics_context::Buffer<[u32; 3]>,
    pub index_start: u32,
    pub index_end: u32,
}

pub fn new_gpu_mesh(
    graphics: &mut koi_graphics_context::GraphicsContext,
    mesh_data: &MeshData,
) -> Result<GPUMesh, ()> {
    // Check that all of the indices point to valid vertices.
    // If this causes performance issues this check could be disabled in the future.
    let len = mesh_data.positions.len();
    for i in mesh_data.indices.iter() {
        assert!(
            !(i[0] as usize >= len || i[1] as usize >= len || i[2] as usize >= len),
            "Mesh indices refer to out of bound vertices: {:?}. Vertex count: {:?} ",
            i,
            mesh_data.positions.len(),
            //  mesh_data.indices
        );
    }

    let triangle_count = mesh_data.indices.len() as u32;

    let texture_coordinates = if !mesh_data.texture_coordinates.is_empty() {
        assert_eq!(mesh_data.texture_coordinates.len(), len);
        Some(graphics.new_buffer(
            &mesh_data.texture_coordinates,
            koi_graphics_context::BufferUsage::Data,
        ))
    } else {
        None
    };
    let normals = if !mesh_data.normals.is_empty() {
        assert_eq!(mesh_data.normals.len(), len);
        Some(graphics.new_buffer(&mesh_data.normals, koi_graphics_context::BufferUsage::Data))
    } else {
        None
    };

    let colors = if !mesh_data.colors.is_empty() {
        assert_eq!(mesh_data.colors.len(), len);
        Some(graphics.new_buffer(&mesh_data.colors, koi_graphics_context::BufferUsage::Data))
    } else {
        None
    };

    Ok(GPUMesh {
        positions: graphics.new_buffer(
            &mesh_data.positions,
            koi_graphics_context::BufferUsage::Data,
        ),
        texture_coordinates,
        normals,
        index_buffer: graphics
            .new_buffer(&mesh_data.indices, koi_graphics_context::BufferUsage::Index),
        colors,
        index_start: 0,
        index_end: triangle_count,
    })
}
