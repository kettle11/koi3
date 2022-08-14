use koi3::*;

pub struct Running;

fn main() {
    App::default().run(|_event, world, resources| {
        if resources.try_get::<Running>().is_none() {
            resources.add(Running);
            world.spawn((
                Transform::new(),
                Camera {
                    clear_color: Some(Color::ORANGE),
                    projection_mode: ProjectionMode::Orthographic {
                        height: 4.0,
                        z_near: -5.0,
                        z_far: 5.0,
                    },
                    ..Default::default()
                },
            ));

            world.spawn((Transform::new(), Mesh::VERTICAL_QUAD, Material::TEST));
        }

        // When a key is pressed change the camera's clear color.
        if resources.get_mut::<Input>().key_down(Key::Space) {
            let (_, camera) = world.query_mut::<&mut Camera>().into_iter().next().unwrap();
            camera.clear_color = Some(Color::ELECTRIC_INDIGO);
        }
    });
}
