use crate::*;
use koi_assets::*;

pub struct AudioSource {
    to_play: Vec<(Handle<Sound>, bool)>,
    playing: Vec<OddioHandle>,
}

impl AudioSource {
    pub fn new() -> Self {
        Self {
            to_play: Vec::new(),
            playing: Vec::new(),
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

    let mut spatial_scene_control = audio_manager
        .spatial_handle
        .control::<oddio::SpatialScene, _>();

    for (_, (transform, audio_source)) in
        world.query_mut::<(&koi_transform::Transform, &mut AudioSource)>()
    {
        for (sound_handle, looped) in audio_source.to_play.drain(..) {
            let sound = sounds.get(&sound_handle);

            let spatial_options = oddio::SpatialOptions {
                position: transform.position.as_array().into(),
                velocity: [0.0, 0.0, 0.0].into(),
                radius: 1.0,
            };

            let oddio_handle = if looped {
                let source = oddio::Cycle::new(sound.frames.clone());
                OddioHandle::SpatialLooped(spatial_scene_control.play(source, spatial_options))
            } else {
                println!("PLAYING SOUND");
                OddioHandle::Spatial(spatial_scene_control.play(
                    oddio::FramesSignal::new(sound.frames.clone(), 0.),
                    spatial_options,
                ))
            };
            audio_source.playing.push(oddio_handle);
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
    fn stop(&mut self) {
        match self {
            OddioHandle::Spatial(s) => s.control::<oddio::Stop<_>, _>(),
            OddioHandle::SpatialLooped(s) => s.control::<oddio::Stop<_>, _>(),
        };
    }
}
