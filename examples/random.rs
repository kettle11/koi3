use koi3::*;
use koi_camera_controls::CameraControls;

pub struct Running;

fn main() {
    App::default().run(|_event, world, resources| {
        if resources.try_get::<Running>().is_none() {
            resources.add(Running);
            world.spawn((
                Transform::new().with_position(Vec3::Z * 10.0),
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
        }
    });
}
