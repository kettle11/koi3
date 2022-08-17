//! This example demonstrates a pattern that may be useful for gamejams
//! or quick prototypes.
//!
//! Within the setup and run function gamestate can be initialized
//! and then used later in the loop without explicitly
//! storing it in a state struct.

use koi3::*;

fn main() {
    App::default().setup_and_run(|world, _resources| {
        world.spawn((
            Transform::new().with_position(Vec3::Z * 3.0),
            Camera {
                clear_color: Some(Color::ORANGE),
                ..Default::default()
            },
        ));

        let entity = world.spawn((Transform::new(), Mesh::VERTICAL_QUAD, Material::UNLIT));

        move |event, world, resources| match event {
            Event::FixedUpdate => {
                let input = resources.get_mut::<Input>();

                let mut movement = Vec3::ZERO;

                if input.key(Key::Left) {
                    movement -= Vec3::X;
                }
                if input.key(Key::Right) {
                    movement += Vec3::X;
                }
                if input.key(Key::Up) {
                    movement += Vec3::Y;
                }
                if input.key(Key::Down) {
                    movement -= Vec3::Y;
                }

                world.get::<&mut Transform>(entity).unwrap().position +=
                    movement.normalized_or_zero() * 0.1;
            }
            _ => {}
        }
    })
}
