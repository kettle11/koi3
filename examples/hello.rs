use koi3::*;

pub struct Running;

fn main() {
    App::default().run(|_event, world, resources| {
        
        if resources.try_get::<Running>().is_none() {
            resources.add(Running);
            world.spawn((
                Transform::new(),
                Camera {
                    clear_color: Some(Color::BLUE),
                    projection_mode: ProjectionMode::Orthographic {
                        height: 2.0,
                        z_near: -5.0,
                        z_far: 5.0,
                    },
                    ..Default::default()
                },
            ));

            world.spawn((Transform::new(), Mesh::VERTICAL_QUAD, Material::TEST));
        }
    });
}
