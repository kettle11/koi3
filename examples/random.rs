use koi3::*;
use koi_camera_controls::CameraControls;

fn main() {
    App::default().setup_and_run(|world, _resources| {
        world.spawn((
            Transform::new().with_position(Vec3::Z * 13.0),
            Camera {
                clear_color: Some(Color::CORNFLOWER_BLUE),
                ..Default::default()
            },
            CameraControls::new(),
        ));

        let mut random = Random::new();
        for _ in 0..100 {
            world.spawn((
                Transform::new().with_position(random.point_in_unit_sphere() * 20.0),
                Mesh::SPHERE,
                Material::UNLIT,
            ));
        }

        |_event, _world, _resources| {}
    });
}
