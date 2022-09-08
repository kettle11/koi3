//! This example demonstrates how to programatically
//! construct an animation. Normally this sort of animation is laoded from a
//! 3D asset like a glTF.
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

        let mut transform_animations = resources.get::<AssetStore<Animation>>();
        let animation = transform_animations.add(Animation::new(vec![AnimationClip {
            animation_curve: animation_curves::smooth_step,
            entity_mapping_index: 0,
            typed_animation_clip: Box::new(TypedAnimationClip::<Transform> {
                set_property: |e: &koi_ecs::EntityRef, v0: &Transform, v1: &Transform, t| {
                    *e.get::<&mut Transform>().unwrap() = v0.interpolate(v1, t)
                },
                key_frames: vec![0.0, 1.0, 2.0],
                values: vec![
                    Transform::new()
                        .with_position(Vec3::X * -2.0)
                        .with_rotation(Quat::from_angle_axis(0.0, Vec3::Y)),
                    Transform::new()
                        .with_position(Vec3::X * 2.0)
                        .with_rotation(Quat::from_angle_axis(std::f32::consts::PI, Vec3::Y)),
                    Transform::new().with_position(Vec3::X * -2.0),
                ],
            }),
        }]));

        let e = world.spawn((Transform::new(), Mesh::VERTICAL_QUAD, Material::UNLIT));
        world.spawn((AnimationPlayer {
            playing_animations: vec![PlayingAnimation {
                time: 0.0,
                entity_mapping: vec![Some(e)],
                animation,
            }],
            animations: Default::default(),
        },));

        move |event, _world, _resources| match event {
            Event::FixedUpdate => {}
            _ => {}
        }
    });
}
