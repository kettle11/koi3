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

        let custom_shader = resources
            .get::<AssetStore<Shader>>()
            .load("examples/assets/custom_shader.glsl");

        let custom_material = resources.get::<AssetStore<Material>>().add(Material {
            shader: custom_shader,
            base_color: Color::AZURE,
            ..Default::default()
        });

        world.spawn((Transform::new(), Mesh::VERTICAL_QUAD, custom_material));
    }

    // When a key is pressed change the camera's clear color.
    if resources.get_mut::<Input>().key_down(Key::Space) {
        let (_, camera) = world.query_mut::<&mut Camera>().into_iter().next().unwrap();
        camera.clear_color = Some(Color::ELECTRIC_INDIGO);
    }
}
