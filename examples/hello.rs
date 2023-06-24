use koi3::*;
use koi_camera_controls::CameraControls;

fn main() {
    App::default()
        .with_resource(InitialSettings {
            color_space: koi_graphics_context::ColorSpace::DisplayP3,
            ..Default::default()
        })
        .setup_and_run(|world, _resources| {
            world.spawn((
                Transform::new().with_position(Vec3::Z * 2.0),
                Camera {
                    clear_color: Some(Color::new_with_colorspace(
                        1.0,
                        0.0,
                        0.0,
                        1.0,
                        color_spaces::DISPLAY_P3,
                    )),
                    ..Default::default()
                },
                CameraControls::new(),
            ));

            // world.spawn((Mesh::SPHERE, Material::UNLIT, Transform::new()));

            world.spawn((Transform::new(), Mesh::VERTICAL_CIRCLE, Material::UNLIT));

            world.spawn((
                Transform::new().with_position(Vec3::Y),
                Mesh::VERTICAL_QUAD,
                Material::UNLIT,
            ));

            // This function will run for major events liked a FixedUpdate occuring
            // and for any input events from the application.
            // See [koi::Event]
            |event, world, resources| match event {
                Event::FixedUpdate => {
                    if resources.get_mut::<Input>().key_down(Key::Space) {
                        let (_, camera) =
                            world.query_mut::<&mut Camera>().into_iter().next().unwrap();
                        camera.clear_color = Some(Color::ELECTRIC_INDIGO);
                    }
                }
                _ => {}
            }
        });
}
