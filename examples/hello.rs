use koi3::*;

pub struct Running;

fn main() {
    App::default().setup_and_run(|world, resources| {
        resources.add(Running);
        world.spawn((
            Transform::new().with_position(Vec3::Z * 2.0),
            Camera {
                clear_color: Some(Color::ORANGE),
                ..Default::default()
            },
        ));

        world.spawn((
            Transform::new(),
            DirectionalLight {
                intensity_illuminance: 1.0,
                color: Color::RED,
            },
        ));

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
