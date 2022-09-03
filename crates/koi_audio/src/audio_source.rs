use crate::{audio_listener::AudioListener, *};
use kmath::*;
use koi_assets::*;

// TODO: Make this a component that can be cloned between worlds
pub struct AudioSource {
    to_play: Vec<(Handle<Sound>, bool)>,
    playing: Vec<OddioHandle>,
    previous_position: Option<Vec3>,
    previous_velocity: Option<Vec3>,
}

impl AudioSource {
    pub fn new() -> Self {
        Self {
            to_play: Vec::new(),
            playing: Vec::new(),
            previous_position: None,
            previous_velocity: None,
        }
    }
    pub fn play_sound(&mut self, sound: &Handle<Sound>) {
        self.to_play.push((sound.clone(), false));
    }
    pub fn play_sound_looped(&mut self, sound: &Handle<Sound>) {
        self.to_play.push((sound.clone(), true));
    }
}

pub fn update_audio_sources(
    _event: &koi_events::Event,
    world: &mut koi_ecs::World,
    resources: &mut koi_resources::Resources,
) {
    let mut audio_manager = resources.get::<AudioManager>();
    let mut sounds = resources.get::<AssetStore<Sound>>();
    let time = resources.get::<koi_time::Time>();

    let mut spatial_scene_control = audio_manager
        .spatial_handle
        .control::<oddio::SpatialScene, _>();

    if let Some((_, (listener_transform, listener))) = world
        .query::<(&koi_transform::GlobalTransform, &mut AudioListener)>()
        .iter()
        .next()
    {
        for (_, (transform, audio_source)) in world
            .query::<(&koi_transform::GlobalTransform, &mut AudioSource)>()
            .iter()
        {
            // First we calculate how much this AudioSource has moved relative to the listener
            // And we also calculate if there was any discontinuity in the movement of the
            // AudioSource.

            fn discontinuous(
                current_p: Vec3,
                previous_p: Option<Vec3>,
                previous_v: Option<Vec3>,
                time_step: f32,
            ) -> bool {
                if let Some(previous_v) = previous_v {
                    if let Some(previous_p) = previous_p {
                        let predicted_position = previous_p + previous_v * time_step;
                        let diff = (predicted_position - current_p).length_squared();

                        // This is a rather arbitrary threshold.
                        if diff < 0.1 {
                            return false;
                        }
                    }
                }
                true
            }

            let discontinuity = discontinuous(
                transform.position,
                audio_source.previous_position,
                audio_source.previous_velocity,
                time.fixed_time_step_seconds as f32,
            ) || discontinuous(
                listener_transform.position,
                listener.previous_position,
                listener.previous_velocity,
                time.fixed_time_step_seconds as f32,
            );

            let velocity_meters_per_second = audio_source
                .previous_position
                .map_or(Vec3::ZERO, |p| transform.position - p)
                / time.fixed_time_step_seconds as f32;

            let listener_velocity_meters_per_second = listener
                .previous_position
                .map_or(Vec3::ZERO, |p| listener_transform.position - p)
                / time.fixed_time_step_seconds as f32;
            let relative_position = transform.position - listener_transform.position;
            let relative_velocity =
                velocity_meters_per_second - listener_velocity_meters_per_second;

            audio_source.previous_position = Some(transform.position);
            audio_source.previous_velocity = Some(velocity_meters_per_second);
            listener.previous_velocity = Some(listener_velocity_meters_per_second);
            listener.previous_position = Some(listener_transform.position);

            for sound in audio_source.playing.iter_mut() {
                sound.set_motion(relative_position, relative_velocity, discontinuity);
            }

            for (sound_handle, looped) in audio_source.to_play.drain(..) {
                let sound = sounds.get(&sound_handle);

                let spatial_options = oddio::SpatialOptions {
                    position: transform.position.as_array().into(),
                    velocity: velocity_meters_per_second.as_array().into(),
                    radius: 1.0,
                };

                let oddio_handle = if looped {
                    let source = oddio::Cycle::new(sound.frames.clone());
                    OddioHandle::SpatialLooped(spatial_scene_control.play(source, spatial_options))
                } else {
                    OddioHandle::Spatial(spatial_scene_control.play(
                        oddio::FramesSignal::new(sound.frames.clone(), 0.),
                        spatial_options,
                    ))
                };
                audio_source.playing.push(oddio_handle);
            }
        }
    }

    sounds.finalize_asset_loads(resources);
    sounds.cleanup_dropped_assets();
}

impl Drop for OddioHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

enum OddioHandle {
    Spatial(oddio::Handle<oddio::Spatial<oddio::Stop<oddio::FramesSignal<f32>>>>),
    SpatialLooped(oddio::Handle<oddio::Spatial<oddio::Stop<oddio::Cycle<f32>>>>),
}

impl OddioHandle {
    fn set_motion(&mut self, position: Vec3, velocity: Vec3, discontinuity: bool) {
        let mut control = match self {
            OddioHandle::Spatial(s) => s.control::<oddio::Spatial<_>, _>(),
            OddioHandle::SpatialLooped(s) => s.control::<oddio::Spatial<_>, _>(),
        };
        control.set_motion(
            position.as_array().into(),
            velocity.as_array().into(),
            discontinuity,
        );
    }

    fn stop(&mut self) {
        match self {
            OddioHandle::Spatial(s) => s.control::<oddio::Stop<_>, _>(),
            OddioHandle::SpatialLooped(s) => s.control::<oddio::Stop<_>, _>(),
        };
    }
}
