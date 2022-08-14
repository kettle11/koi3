use kgraphics::GraphicsContextTrait;
use kmath::*;

pub struct Mesh {
    pub(crate) gpu_mesh: Option<GPUMesh>,
    pub mesh_data: Option<MeshData>,
    pub bounding_box: Option<Box3>,
}

impl Mesh {
    pub fn new(graphics: &mut kgraphics::GraphicsContext, mesh_data: MeshData) -> Self {
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

    pub fn update_mesh_on_gpu(&mut self, graphics: &mut kgraphics::GraphicsContext) {
        if let Some(gpu_mesh) = self.gpu_mesh.take() {
            gpu_mesh.delete(graphics);
        }
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

#[derive(Clone)]
pub struct GPUMesh {
    pub positions: kgraphics::DataBuffer<kmath::Vec3>,
    pub texture_coordinates: Option<kgraphics::DataBuffer<kmath::Vec2>>,
    pub normals: Option<kgraphics::DataBuffer<kmath::Vec3>>,
    pub colors: Option<kgraphics::DataBuffer<kmath::Vec4>>,
    pub index_buffer: kgraphics::IndexBuffer,
    pub index_start: u32,
    pub index_end: u32,
}

pub fn new_gpu_mesh(
    graphics: &mut kgraphics::GraphicsContext,
    mesh_data: &MeshData,
) -> Result<GPUMesh, kgraphics::GraphicsError> {
    // Check that all of the indices point to valid vertices.
    // If this causes performance issues this check could be disabled in the future.
    let len = mesh_data.positions.len();
    for i in mesh_data.indices.iter() {
        assert!(
            !(i[0] as usize >= len || i[1] as usize >= len || i[2] as usize >= len),
            "Mesh indices refer to out of bound vertices: {:?}. Vertex count: {:?}",
            i,
            mesh_data.positions.len()
        );
    }

    let triangle_count = mesh_data.indices.len() as u32;

    // Flatten the index buffer
    let index_buffer: &[u32] = unsafe {
        std::slice::from_raw_parts(
            mesh_data.indices.as_ptr() as *const u32,
            mesh_data.indices.len() * 3,
        )
    };

    let texture_coordinates = if !mesh_data.texture_coordinates.is_empty() {
        assert_eq!(mesh_data.texture_coordinates.len(), len);
        Some(graphics.new_data_buffer(&mesh_data.texture_coordinates)?)
    } else {
        None
    };
    let normals = if !mesh_data.normals.is_empty() {
        assert_eq!(mesh_data.normals.len(), len);
        Some(graphics.new_data_buffer(&mesh_data.normals)?)
    } else {
        None
    };

    let colors = if !mesh_data.colors.is_empty() {
        assert_eq!(mesh_data.colors.len(), len);
        Some(graphics.new_data_buffer(&mesh_data.colors)?)
    } else {
        None
    };

    Ok(GPUMesh {
        positions: graphics.new_data_buffer(&mesh_data.positions)?,
        texture_coordinates,
        normals,
        index_buffer: graphics.new_index_buffer(index_buffer)?,
        colors,
        index_start: 0,
        index_end: triangle_count,
    })
}

impl GPUMesh {
    pub fn delete(self, graphics: &mut kgraphics::GraphicsContext) {
        let GPUMesh {
            positions,
            normals,
            index_buffer,
            texture_coordinates,
            colors,
            index_start: _,
            index_end: _,
        } = self;
        graphics.delete_data_buffer(positions);
        graphics.delete_index_buffer(index_buffer);

        if let Some(d) = normals {
            graphics.delete_data_buffer(d);
        }
        if let Some(d) = texture_coordinates {
            graphics.delete_data_buffer(d);
        }
        if let Some(d) = colors {
            graphics.delete_data_buffer(d);
        }
    }
}
