use crate::{audio_listener::AudioListener, *};
use kmath::*;
use koi_assets::*;
use oddio::Frames;

struct ToPlayCustom {
    sound_handle: Option<Handle<Sound>>,
    produce_oddio_filter: Box<
        dyn FnMut(Option<std::sync::Arc<Frames<f32>>>) -> Box<dyn oddio::Seek<Frame = f32> + Send>
            + Send
            + Sync,
    >,
}

// TODO: Make this a component that can be cloned between worlds
pub struct AudioSource {
    to_play_spatial: Vec<ToPlayCustom>,
    playing: Vec<OddioHandle>,
    previous_position: Option<Vec3>,
    previous_velocity: Option<Vec3>,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioSource {
    pub fn new() -> Self {
        Self {
            to_play_spatial: Vec::new(),
            playing: Vec::new(),
            previous_position: None,
            previous_velocity: None,
        }
    }
    pub fn play_sound(&mut self, sound: &Handle<Sound>) {
        self.to_play_spatial.push(ToPlayCustom {
            sound_handle: Some(sound.clone()),
            produce_oddio_filter: Box::new(move |frames: Option<std::sync::Arc<Frames<f32>>>| {
                Box::new(oddio::FramesSignal::new(frames.unwrap(), 0.0))
            }),
        });
    }
    pub fn play_sound_looped(&mut self, sound: &Handle<Sound>) {
        self.to_play_spatial.push(ToPlayCustom {
            sound_handle: Some(sound.clone()),
            produce_oddio_filter: Box::new(move |frames: Option<std::sync::Arc<Frames<f32>>>| {
                Box::new(oddio::Cycle::new(frames.unwrap()))
            }),
        });
    }

    pub fn play_sound_custom<F: oddio::Seek<Frame = f32> + Send + 'static>(
        &mut self,
        sound: Option<&Handle<Sound>>,
        mut produce_source: impl FnMut(Option<std::sync::Arc<Frames<f32>>>) -> F + Send + Sync + 'static,
    ) {
        self.to_play_spatial.push(ToPlayCustom {
            sound_handle: sound.map(|s| s.clone()),
            produce_oddio_filter: Box::new(move |frames: Option<std::sync::Arc<Frames<f32>>>| {
                Box::new(produce_source(frames))
            }),
        });
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

    if let Some((_, (listener_transform, listener))) = world
        .query::<(&koi_transform::GlobalTransform, &mut AudioListener)>()
        .iter()
        .next()
    {
        let q: [f32; 4] = listener_transform.rotation.into();
        let mut spatial_scene_control = audio_manager
            .spatial_handle
            .control::<oddio::SpatialScene, _>();
        spatial_scene_control.set_listener_rotation(q.into());

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

            audio_source.playing.retain_mut(|sound| {
                sound.set_motion(relative_position, relative_velocity, discontinuity);
                if sound.is_done() {
                    sound.stop();
                }
                !sound.is_done()
            });

            // TODO: This approach to continuosly attempting to initialize placeholder sounds
            // is not great.
            // Sound effects / music should skip ahead based on how long it took to load them.

            let spatial_options = oddio::SpatialOptions {
                position: relative_position.as_array().into(),
                velocity: relative_velocity.as_array().into(),
                radius: 1.0,
            };

            audio_source.to_play_spatial.retain_mut(
                |ToPlayCustom {
                     sound_handle,
                     produce_oddio_filter,
                 }| {
                    if let Some(sound_handle) = sound_handle {
                        if !sounds.is_placeholder(&sound_handle) {
                            let sound = sounds.get(&sound_handle);
                            let source = produce_oddio_filter(Some(sound.frames.clone()));
                            let oddio_handle = spatial_scene_control.play(source, spatial_options);
                            audio_source
                                .playing
                                .push(OddioHandle::Spatial(oddio_handle));
                            false
                        } else {
                            true
                        }
                    } else {
                        let source = produce_oddio_filter(None);
                        let oddio_handle = spatial_scene_control.play(source, spatial_options);
                        audio_source
                            .playing
                            .push(OddioHandle::Spatial(oddio_handle));
                        true
                    }
                },
            );
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
    Spatial(oddio::Handle<oddio::Spatial<oddio::Stop<Box<dyn oddio::Seek<Frame = f32> + Send>>>>),
}

impl OddioHandle {
    fn set_motion(&mut self, position: Vec3, velocity: Vec3, discontinuity: bool) {
        let mut control = match self {
            OddioHandle::Spatial(s) => s.control::<oddio::Spatial<_>, _>(),
        };
        control.set_motion(
            position.as_array().into(),
            velocity.as_array().into(),
            discontinuity,
        );
    }

    fn stop(&mut self) {
        let control = match self {
            OddioHandle::Spatial(s) => s.control::<oddio::Stop<_>, _>(),
        };
        control.stop();
    }

    fn is_done(&mut self) -> bool {
        let control = match self {
            OddioHandle::Spatial(s) => s.control::<oddio::Stop<_>, _>(),
        };
        control.is_stopped()
    }
}
