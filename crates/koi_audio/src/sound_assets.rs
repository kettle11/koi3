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

        Sound::from_file_bytes(&bytes, extension, settings.scale)
    }
    resources.add(koi_assets::AssetStore::new_with_load_functions(
        Sound::new_from_slice(&[0.0]),
        load,
        |sound, _settings, _resources| Some(sound),
    ));
}
