use koi3::*;

fn main() {
    App::default().setup_and_run(|world, resources| {
        world.spawn((
            Transform::new().with_position(Vec3::Z * 2.0),
            Camera {
                clear_color: Some(Color::ORANGE),
                ..Default::default()
            },
        ));

        world.spawn((Transform::new(), Mesh::VERTICAL_QUAD, Material::UNLIT));

        let sound = resources
            .get::<AssetStore<Sound>>()
            .load("assets/bell.wav", Default::default());
        let audio_source = AudioSource::new();
        // audio_source.play_sound(&sound);

        let audio_source_entity = world.spawn((Transform::new(), audio_source));
        // This function will run for major events liked a FixedUpdate occuring
        // and for any input events from the application.
        // See [koi::Event]
        move |event, world, resources| match event {
            Event::FixedUpdate => {
                if resources.get_mut::<Input>().key_down(Key::Space) {
                    world
                        .get::<&mut AudioSource>(audio_source_entity)
                        .unwrap()
                        .play_sound(&sound);
                }
            }
            _ => {}
        }
    });
}
