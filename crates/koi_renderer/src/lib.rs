mod koi_integration;
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

pub fn raycast_scene(
    world: &koi_ecs::World,
    resources: &koi_resources::Resources,
    ray: kmath::Ray3,
) -> Option<(kmath::Vec3, koi_ecs::Entity)> {
    let meshes = resources.get::<koi_assets::AssetStore<Mesh>>();
    let mut entities = world.query::<(&GlobalTransform, &koi_assets::Handle<Mesh>)>();
    raycast_entities(ray, &*meshes, entities.iter())
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
) -> Option<(kmath::Vec3, koi_ecs::Entity)> {
    let mut closest_value = f32::MAX;
    let mut intersected_entity = None;
    for (entity, (transform, mesh_handle)) in entities {
        let mesh = meshes.get(mesh_handle);
        if let Some(bounding_box) = mesh.bounding_box {
            if let Some(mesh_data) = mesh.mesh_data.as_ref() {
                // This inverse will make this operation more slow
                let inverse_model = transform.local_to_world().inversed();
                let ray = inverse_model.transform_ray(ray);
                if let Some(v) = kmath::intersections::ray_with_bounding_box(ray, bounding_box) {
                    if v < closest_value {
                        if let Some(v) = kmath::intersections::ray_with_mesh(
                            ray,
                            &mesh_data.positions,
                            &mesh_data.indices,
                        ) {
                            if v < closest_value {
                                closest_value = v;
                                intersected_entity = Some(entity)
                            }
                        }
                    }
                }
            }
        }
    }
    intersected_entity.map(|i| (ray.get_point(closest_value), i))
}
