mod koi_integration;
use kmath::Ray3;
pub use koi_integration::*;

mod camera;
pub use camera::*;

mod light;
use koi_transform::GlobalTransform;
pub use light::*;

mod mesh;
pub use mesh::*;

mod mesh_constants;
pub use mesh_constants::*;

mod shader;
pub use shader::*;

mod shader_constants;
pub use shader_constants::*;

mod material;
pub use material::*;

mod material_constants;
pub use material_constants::*;

mod texture;
pub use texture::*;

mod texture_constants;
pub use texture_constants::*;

mod cube_map;
pub use cube_map::*;

pub use kcolor::*;
pub use koi_graphics_context;

mod renderer;
pub use renderer::*;

mod shader_parser;
mod spherical_harmonics;

mod morphable_mesh;
pub use morphable_mesh::*;

// mod specular_precompute;

mod light_probe;
pub use light_probe::*;

#[derive(Debug, Clone)]
pub struct RaycastSceneResult {
    pub position: kmath::Vec3,
    pub normal: Option<kmath::Vec3>,
    pub mesh_tri_index: usize,
    pub entity: koi_ecs::Entity,
}

pub fn raycast_screen(
    world: &koi_ecs::World,
    resources: &koi_resources::Resources,
    camera_entity: koi_ecs::Entity,
    x: f32,
    y: f32,
) -> Option<RaycastSceneResult> {
    use std::ops::Deref;

    let window = resources.get::<kapp::Window>();
    let (window_width, window_height) = window.size();

    let entity_ref = world.entity(camera_entity).unwrap();
    let camera = entity_ref.get::<&Camera>().unwrap();
    let camera_transform = entity_ref.get::<&GlobalTransform>().unwrap();

    let ray = camera.view_to_ray(
        camera_transform.deref(),
        x,
        y,
        window_width as f32,
        window_height as f32,
    );
    raycast_scene(world, resources, ray)
}

pub fn raycast_scene(
    world: &koi_ecs::World,
    resources: &koi_resources::Resources,
    ray: kmath::Ray3,
) -> Option<RaycastSceneResult> {
    let meshes = resources.get::<koi_assets::AssetStore<Mesh>>();
    let mut entities = world.query::<(&GlobalTransform, &koi_assets::Handle<Mesh>)>();
    raycast_entities(ray, &*meshes, entities.iter())
}

/// Gets the world ray of the pointer given an x and y position in screen pixels
/// and a camera. This assumes there is one window.
pub fn get_pointer_world_ray(
    world: &koi_ecs::World,
    resources: &koi_resources::Resources,
    camera_entity: koi_ecs::Entity,
    x: f32,
    y: f32,
) -> Option<Ray3> {
    use std::ops::Deref;

    let window = resources.get::<kapp::Window>();
    let (window_width, window_height) = window.size();

    let entity_ref = world.entity(camera_entity).ok()?;
    let camera = entity_ref.get::<&Camera>()?;
    let camera_transform = entity_ref.get::<&GlobalTransform>()?;

    Some(camera.view_to_ray(
        camera_transform.deref(),
        x,
        y,
        window_width as f32,
        window_height as f32,
    ))
}

pub fn raycast_entities<'a>(
    ray: kmath::Ray3,
    meshes: &koi_assets::AssetStore<Mesh>,
    entities: impl Iterator<
        Item = (
            koi_ecs::Entity,
            (&'a GlobalTransform, &'a koi_assets::Handle<Mesh>),
        ),
    >,
) -> Option<RaycastSceneResult> {
    let mut closest_value = f32::MAX;
    let mut intersected_entity = None;
    let mut intersected_tri = 0;
    let mut normal = None;

    for (entity, (transform, mesh_handle)) in entities {
        let mesh = meshes.get(mesh_handle);
        if let Some(bounding_box) = mesh.bounding_box {
            if let Some(mesh_data) = mesh.mesh_data.as_ref() {
                let transform = transform.local_to_world();

                // This inverse will make this operation more slow
                let inverse_model = transform.inversed();
                let ray = inverse_model.transform_ray(ray);
                if let Some(v) = kmath::intersections::ray_with_bounding_box(ray, bounding_box) {
                    if v < closest_value {
                        if let Some(kmath::intersections::RayWithMeshResult {
                            distance: v,
                            tri_index,
                            barycentric_coordinates,
                        }) = kmath::intersections::ray_with_mesh(
                            ray,
                            &mesh_data.positions,
                            &mesh_data.indices,
                        ) {
                            if v < closest_value {
                                closest_value = v;
                                intersected_tri = tri_index;
                                intersected_entity = Some(entity);

                                normal = None;

                                let tri = mesh_data.indices[tri_index];
                                if let Some(n0) = mesh_data.normals.get(tri[0] as usize) {
                                    if let Some(n1) = mesh_data.normals.get(tri[1] as usize) {
                                        if let Some(n2) = mesh_data.normals.get(tri[2] as usize) {
                                            let (u, v, w) = barycentric_coordinates.into();
                                            let local_space_normal = *n0 * u + *n1 * v + *n2 * w;
                                            let world_space_normal =
                                                transform.transform_vector(local_space_normal);

                                            normal = Some(world_space_normal);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    intersected_entity.map(|entity| RaycastSceneResult {
        position: ray.get_point(closest_value),
        entity,
        mesh_tri_index: intersected_tri,
        normal,
    })
}
