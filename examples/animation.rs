use koi3::*;
use koi_camera_controls::CameraControls;

fn main() {
    App::default().setup_and_run(|world, resources| {
        world.spawn((
            Transform::new().with_position(Vec3::Z * 5.0),
            Camera {
                clear_color: Some(Color::ORANGE),
                ..Default::default()
            },
            CameraControls::new(),
        ));

        let mut transform_animations = resources.get::<AssetStore<Animation<Transform>>>();
        let animation = transform_animations.add(Animation {
            key_frames: vec![
                KeyFrame {
                    timestamp: 0.0,
                    value: Transform::new()
                        .with_position(Vec3::X * -2.0)
                        .with_rotation(Quat::from_angle_axis(0.0, Vec3::Y)),
                },
                KeyFrame {
                    timestamp: 1.0,
                    value: Transform::new()
                        .with_position(Vec3::X * 2.0)
                        .with_rotation(Quat::from_angle_axis(std::f32::consts::PI, Vec3::Y)),
                },
                KeyFrame {
                    timestamp: 2.0,
                    value: Transform::new().with_position(Vec3::X * -2.0),
                },
            ],
            animation_curve: animation_curves::smooth_step,
        });

        world.spawn((
            Transform::new(),
            Mesh::VERTICAL_QUAD,
            Material::UNLIT,
            AnimationPlayer {
                time: 0.0,
                animation,
            },
        ));

        // This function will run for major events liked a FixedUpdate occuring
        // and for any input events from the application.
        // See [koi::Event]
        move |event, world, resources| match event {
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
