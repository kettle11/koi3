use crate::*;

impl koi_assets::AssetTrait for Sound {
    type Settings = SoundSettings;
}

#[derive(Clone)]
pub struct SoundSettings {
    pub scale: f32,
}
impl Default for SoundSettings {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

pub fn initialize_sound_assets(resources: &mut koi_resources::Resources) {
    async fn load(path: String, settings: SoundSettings) -> Option<Sound> {
        let bytes = koi_fetch::fetch_bytes(&path)
            .await
            .unwrap_or_else(|_| panic!("Failed to open file: {}", path));
        let extension = std::path::Path::new(&path)
            .extension()
            .and_then(std::ffi::OsStr::to_str);

        match extension {
            Some("wav") => {
                let mut sound = kaudio::load_wav_from_bytes(&bytes).unwrap();

                // Apply scale
                sound.data.iter_mut().for_each(|s| *s *= settings.scale);

                if sound.channels > 1 {
                    let channels = sound.channels as f32;
                    // Reduce the sound to mono by taking the average of the channels.
                    sound.data = sound
                        .data
                        .chunks(sound.channels as usize)
                        .map(|d| d.iter().sum::<f32>() / channels)
                        .collect();
                }
                Some(Sound::new_from_iter(sound.data.into_iter()))
            }
            _ => panic!("Unsupported audio format"),
        }
    }
    resources.add(koi_assets::AssetStore::new_with_load_functions(
        Sound::new_from_slice(&[0.0]),
        load,
        |sound, _settings, _resources| Some(sound),
    ));
}
