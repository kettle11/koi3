use koi3::*;

fn main() {
    App::default().setup_and_run(|world, resources| {
        world.spawn((
            Transform::new().with_position(Vec3::Z * 2.0),
            Camera {
                clear_color: Some(Color::ORANGE),
                exposure: Exposure::EV100(6.0),
                ..Default::default()
            },
        ));

        world.spawn((Transform::new(), DirectionalLight::OFFICE_LIGHTING));

        let new_texture = resources.get::<AssetStore<Texture>>().load(
            "examples/assets/characters.png",
            kgraphics::TextureSettings::default(),
        );
        let new_material = resources.get::<AssetStore<Material>>().add(Material {
            shader: Shader::UNLIT,
            base_color_texture: Some(new_texture),
            ..Default::default()
        });

        world.spawn((Transform::new(), Mesh::VERTICAL_QUAD, new_material));

        // This function will run for major events liked a FixedUpdate occuring
        // and for any input events from the application.
        // See [koi::Event]
        |event, world, resources| match event {
            Event::FixedUpdate => {
                if resources.get_mut::<Input>().key_down(Key::Space) {
                    let (_, camera) = world.query_mut::<&mut Camera>().into_iter().next().unwrap();
                    camera.clear_color = Some(Color::ELECTRIC_INDIGO);
                }
            }
            _ => {}
        }
    });
}
