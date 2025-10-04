use koi3::*;

fn main() {
    App::default().setup_and_run(|world, resources| {
        world.spawn((
            Transform::new().with_position(Vec3::Z * 2.0),
            Camera {
                clear_color: Some(Color::ORANGE),
                ..Default::default()
            },
            AudioListener::new(),
        ));

        world.spawn((Transform::new(), Mesh::VERTICAL_QUAD, Material::UNLIT));

        let sound =
            Sound::from_file_bytes(include_bytes!("../assets/explosion.wav"), Some("wav"), 1.0)
                .unwrap();

        let mut cutoff_hz = 300.0;
        let mut lowpass_audio_handle =
            resources
                .get::<AudioManager>()
                .play_one_shot_oddio(oddio::MonoToStereo::new(
                    low_pass_filter::LowpassFilter::new(
                        cutoff_hz,
                        oddio::Cycle::new(sound.frames.clone()),
                    ),
                ));
        // ;
        let sound = resources
            .get::<AssetStore<Sound>>()
            .load("assets/piano_loop.wav", Default::default());
        let mut audio_source = AudioSource::new();
        /*
        audio_source.play_sound_custom(Some(&sound), |frames| {
            low_pass_filter::LowpassFilter::new(700.0, oddio::Cycle::new(frames.unwrap()))
        });
        */
        // audio_source.play_sound(&sound);

        let change_rate = 50.0;
        let audio_source_entity = world.spawn((Transform::new(), audio_source));
        // This function will run for major events liked a FixedUpdate occuring
        // and for any input events from the application.
        // See [koi::Event]
        move |event, world, resources| match event {
            Event::FixedUpdate => {
                if resources.get_mut::<Input>().key(Key::Up) {
                    cutoff_hz += change_rate;
                    cutoff_hz = cutoff_hz.clamp(50.0, 20_000.0);

                    println!("CUTOFF HZ: {:?}", cutoff_hz);
                    lowpass_audio_handle
                        .control::<low_pass_filter::LowpassFilter<_>, _>()
                        .set_cutoff_hz(cutoff_hz);
                }

                if resources.get_mut::<Input>().key(Key::Down) {
                    cutoff_hz -= change_rate;
                    cutoff_hz = cutoff_hz.clamp(50.0, 20_000.0);
                    println!("CUTOFF HZ: {:?}", cutoff_hz);
                    lowpass_audio_handle
                        .control::<low_pass_filter::LowpassFilter<_>, _>()
                        .set_cutoff_hz(cutoff_hz);
                }
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
