mod sound;
pub use sound::*;

mod audio_listener;
pub use audio_listener::*;

use crate::sound_assets::initialize_sound_assets;

mod audio_source;
mod sound_assets;

pub mod low_pass_filter;

pub use audio_source::*;
pub use oddio;

pub fn initialize_plugin(resources: &mut koi_resources::Resources) {
    const QUIET_AMPLITUDE: f32 = 0.001;

    // This top level mixer ensures sound never goes too loud and also dynamically adjusts
    // volume based on the volume of the playing sources.
    // Sounds can be played at the top level to ensure no spatialization occurs.

    let mixer = oddio::Reinhard::new(oddio::Adapt::new(
        oddio::Mixer::<[f32; 2]>::new(),
        QUIET_AMPLITUDE / 2.0f32.sqrt(),
        oddio::AdaptOptions {
            tau: 0.1,
            max_gain: 1.0,
            low: 0.1 / 2.0f32.sqrt(),
            high: 0.9 / 2.0f32.sqrt(),
        },
    ));
    let (mut mixer_handle, mixer_signal) = oddio::split(mixer);

    // Mix spatialized audio into the scene mix.
    let (spatial_handle, spatial_signal) = oddio::split(oddio::SpatialScene::new());
    mixer_handle
        .control::<oddio::Mixer<_>, _>()
        .play(spatial_signal);

    kaudio::begin_audio_thread(move |samples, _info| {
        let frames = oddio::frame_stereo(samples);
        oddio::run(&mixer_signal, kaudio::SAMPLE_RATE as _, frames);
    });

    initialize_sound_assets(resources);

    resources.add(AudioManager {
        spatial_handle,
        non_spatial_handle: mixer_handle,
    });

    resources
        .get_mut::<koi_events::EventHandlers>()
        .add_handler(
            koi_events::Event::PostFixedUpdate,
            audio_source::update_audio_sources,
        );
}

pub struct AudioManager {
    pub spatial_handle: oddio::Handle<oddio::SpatialScene>,
    pub non_spatial_handle: oddio::Handle<oddio::Reinhard<oddio::Adapt<oddio::Mixer<[f32; 2]>>>>,
}

impl AudioManager {
    pub fn play_one_shot(&mut self, sound: &Sound) {
        self.non_spatial_handle
            .control::<oddio::Mixer<_>, _>()
            .play(oddio::MonoToStereo::new(oddio::FramesSignal::new(
                sound.frames.clone(),
                0.0,
            )));
    }

    pub fn play_one_shot_with_speed(&mut self, sound: &Sound, speed: f32) {
        use oddio::Filter;
        let mut signal = oddio::Speed::new(oddio::MonoToStereo::new(oddio::FramesSignal::new(
            sound.frames.clone(),
            0.0,
        )));
        signal.control::<oddio::Speed<_>, _>().set_speed(speed);
        self.non_spatial_handle
            .control::<oddio::Mixer<_>, _>()
            .play(signal);
    }

    pub fn play_one_shot_oddio<S: oddio::Signal<Frame = [f32; 2]> + Send + 'static>(
        &mut self,
        signal: S,
    ) -> oddio::Handle<oddio::Stop<S>> {
        self.non_spatial_handle
            .control::<oddio::Mixer<_>, _>()
            .play(signal)
    }
}
