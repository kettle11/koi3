use koi3::*;

pub struct Running;

fn main() {
    App::default().setup_and_run(|world, resources| {
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
        let custom_shader = resources.get::<AssetStore<Shader>>().load(
            "examples/assets/custom_shader.glsl",
            ShaderSettings::default(),
        );

        // Create a material that uses the custom shader
        let custom_material = resources.get::<AssetStore<Material>>().add(Material {
            shader: custom_shader,
            ..Default::default()
        });

        // Spawn an entity that references the custom material.
        world.spawn((Transform::new(), Mesh::VERTICAL_QUAD, custom_material));
        |event, _world, resources| {
            match event {
                Event::FixedUpdate => {
                    // When a key is pressed reload all shaders that were loaded from a path.
                    if resources.get_mut::<Input>().key_down(Key::Space) {
                        resources.get::<AssetStore<Shader>>().reload();
                    }
                }
                _ => {}
            }
        }
    });
}
