use koi3::*;

fn main() {
    App::default().setup_and_run(|world, _resources| {
        world.spawn((
            Transform::new().with_position(Vec3::Z * 2.0),
            Camera {
                clear_color: Some(Color::ORANGE),
                exposure: Exposure::EV100(6.0),
                ..Default::default()
            },
        ));

        world.spawn((Transform::new(), DirectionalLight::OFFICE_LIGHTING));

        world.spawn((
            Transform::new(),
            Mesh::VERTICAL_QUAD,
            Material::PHYSICALLY_BASED,
        ));

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
