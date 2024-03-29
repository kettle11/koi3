//! Camera controls to be used by the editor or to quickly get a 3D camera up and running.

use kapp_platform_common::*;
use kmath::*;
use koi_renderer::Camera;

pub fn initialize_plugin(resources: &mut koi_resources::Resources) {
    resources
        .get_mut::<koi_events::EventHandlers>()
        .add_handler(koi_events::Event::FixedUpdate, update_camera_controls);
}

#[derive(Clone)]
pub enum CameraControlsMode {
    Fly,
    Orbit,
}

#[derive(Clone)]
pub struct CameraControls {
    velocity: Vec3,
    pub max_speed: f32,
    pub rotation_sensitivity: f32,
    pub mode: CameraControlsMode,
    pub rotate_button: PointerButton,
    pub panning_mouse_button: Option<PointerButton>,
    pub panning_scale: f32,
    pub touch_rotate_enabled: bool,
    pub enabled: bool,
    pub orbit_target: Vec3,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraControls {
    pub fn new() -> Self {
        Self {
            velocity: Vec3::ZERO,
            max_speed: 10.0,
            rotation_sensitivity: 1.5,
            mode: CameraControlsMode::Fly,
            rotate_button: PointerButton::Secondary,
            panning_mouse_button: Some(PointerButton::Auxillary),
            panning_scale: 1.0,
            touch_rotate_enabled: true,
            enabled: true,
            orbit_target: Vec3::ZERO,
        }
    }

    pub fn new_with_mode(mode: CameraControlsMode) -> Self {
        let mut camera_controls = Self::new();
        camera_controls.mode = mode;
        camera_controls
    }
}

pub fn update_camera_controls(
    _event: &koi_events::Event,
    world: &mut koi_ecs::World,
    resources: &mut koi_resources::Resources,
) {
    let input = resources.get::<koi_input::Input>();
    let time = resources.get::<koi_time::Time>();

    /*
      input: &Input,
    time: &Time,
    mut query: Query<(&mut CameraControls, &mut Camera, &mut Transform)>, */
    let query = world.query_mut::<(
        &mut CameraControls,
        &mut Camera,
        &mut koi_transform::Transform,
    )>();
    for (_, (controls, camera, transform)) in query.into_iter() {
        if !controls.enabled {
            continue;
        }
        let (x, y) = input.mouse_motion();
        let difference: Vec2 = Vec2::new(x as f32, y as f32) / 1000.;

        let mut direction = Vec3::ZERO;

        if input.key(Key::W) {
            direction += transform.forward();
        }

        if input.key(Key::S) {
            direction += transform.back();
        }

        if input.key(Key::A) {
            direction += transform.left();
        }

        if input.key(Key::D) {
            direction += transform.right();
        }

        // Up and down controls
        if input.key(Key::E) {
            direction += transform.up();
        }

        if input.key(Key::Q) {
            direction += transform.down();
        }

        /*
        // Switch quickly to top
        if input.key(Key::Digit1) {
            transform.rotation =
                Quaternion::from_angle_axis(-core::f32::consts::TAU * 0.25, Vec3::X);
        }
        */

        if direction != Vec3::ZERO {
            controls.velocity = direction.normalized() * controls.max_speed;
        } else {
            controls.velocity = Vec3::ZERO;
        }

        if controls.velocity.length() > controls.max_speed {
            controls.velocity = controls.velocity.normalized() * controls.max_speed;
        }

        // Rotation
        let (mut pitch, mut yaw, rotating) = if input.pointer_button(controls.rotate_button) {
            let scale = 4.0;
            (-difference[1] * scale, -difference[0] * scale, true)
        } else {
            (0.0, 0.0, false)
        };

        let mut pan = Vec2::ZERO;

        // Panning
        //  if input.key(Key::LeftShift) || input.key(Key::RightShift) || input.key(Key::Shift) {
        //      let scale = 0.005;
        //      pitch = -input.scroll().1 as f32 * scale;
        //      yaw = -input.scroll().0 as f32 * scale;
        //  } else {
        let scale = 0.0125;
        pan.x -= -input.scroll().0 as f32 * scale;
        pan.y -= -input.scroll().1 as f32 * scale;
        // };

        if controls.touch_rotate_enabled && input.touch_state.touches.len() == 1 {
            if let Some((_, touch)) = input.touch_state.touches.iter().next() {
                let diff = touch.delta();
                pitch -= diff.y / 400.;
                yaw -= diff.x / 400.;
            }
        }

        if let Some(panning_mouse_button) = controls.panning_mouse_button {
            if input.pointer_button(panning_mouse_button) {
                let scale = controls.panning_scale * 10.0;
                pan += difference * scale;
            }
        }

        pan += input.two_finger_pan();

        // On Macs there are sometimes extra large-ish scroll events sent when ending a pan with a two finger click.
        // This would result in the camera jerking. This check avoids that.
        if rotating {
            pan = Vec2::ZERO;
        }

        let left = transform.left();
        let up = transform.up();
        let offset = left * pan.x + up * pan.y;

        match &mut controls.mode {
            CameraControlsMode::Orbit => {
                controls.orbit_target += offset;
                transform.position += offset;
            }
            _ => {
                transform.position += offset;
            }
        };

        let pinch = input.pinch();

        match &mut controls.mode {
            CameraControlsMode::Fly => {
                /*
                let pointer_position = input.pointer_position();
                let zoom_direction = camera.view_to_ray(
                    transform,
                    pointer_position.0 as f32,
                    pointer_position.1 as f32,
                );
                transform.position += zoom_direction.direction * pinch * 3.;
                */

                let rotation_pitch = Quat::from_yaw_pitch_roll(0., pitch, 0.);
                let rotation_yaw = Quat::from_yaw_pitch_roll(yaw, 0., 0.);

                transform.rotation = rotation_yaw * transform.rotation * rotation_pitch;
                transform.position += controls.velocity * time.fixed_time_step_seconds as f32;
                controls.orbit_target += controls.velocity * time.fixed_time_step_seconds as f32;
            }
            CameraControlsMode::Orbit => {
                let diff_here = transform.position - controls.orbit_target;

                match &mut camera.projection_mode {
                    koi_renderer::ProjectionMode::Orthographic { height, .. } => {
                        let new_height = *height - pinch * 3.0;
                        *height = new_height;
                    }
                    _ => {
                        transform.position +=
                            transform.forward() * (pinch * 3.).min(diff_here.length());
                    }
                }

                let rotation_pitch = Quat::from_yaw_pitch_roll(0., pitch, 0.);
                let rotation_yaw = Quat::from_yaw_pitch_roll(yaw, 0., 0.);

                let diff = transform.position - controls.orbit_target;
                let diff_length = diff.length();

                let rotation = rotation_yaw * transform.rotation * rotation_pitch;

                let new_direction = rotation * -Vec3::Z;
                let new_up = rotation * Vec3::Y;

                controls.orbit_target += controls.velocity * time.fixed_time_step_seconds as f32;

                transform.position = controls.orbit_target - new_direction * diff_length;
                transform.rotation = Quat::from_forward_up(new_direction, new_up);
            }
        }
    }
}
