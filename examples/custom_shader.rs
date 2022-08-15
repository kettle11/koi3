use koi3::*;

pub struct Running;

fn main() {
    App::default().run(run_loop);
}

fn run_loop(_event: &Event, world: &mut World, resources: &mut Resources) {
    if resources.try_get::<Running>().is_none() {
        resources.add(Running);

        // Spawn a camera
        world.spawn((
            Transform::new().with_position(Vec3::Z * 2.0),
            Camera {
                clear_color: Some(Color::ORANGE),
                ..Default::default()
            },
        ));

        // Load a custom shader from a path
        let custom_shader = resources
            .get::<AssetStore<Shader>>()
            .load("examples/assets/custom_shader.glsl");

        // Create a material that uses the custom shader
        let custom_material = resources.get::<AssetStore<Material>>().add(Material {
            shader: custom_shader,
            base_color: Color::AZURE,
            ..Default::default()
        });

        // Spawn an entity that references that custom shader.
        world.spawn((Transform::new(), Mesh::VERTICAL_QUAD, custom_material));
    }
}
