/// A sound source.
pub struct Sound {
    pub frames: std::sync::Arc<oddio::Frames<f32>>,
}

impl Sound {
    /// The sample rate must be 44100
    pub fn new_from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = f32>,
        I::IntoIter: ExactSizeIterator,
    {
        let frames = oddio::Frames::from_iter(kaudio::SAMPLE_RATE as _, iter);
        Sound { frames }
    }

    /// The sample rate must be 44100
    pub fn new_from_slice(slice: &[f32]) -> Self {
        let frames = oddio::Frames::from_slice(kaudio::SAMPLE_RATE as _, slice);
        Sound { frames }
    }

    pub fn from_file_bytes(bytes: &[u8], extension: Option<&str>, scale: f32) -> Option<Self> {
        match extension {
            Some("wav") => {
                let mut sound = kaudio::load_wav_from_bytes(bytes).unwrap();

                // Apply scale
                sound.data.iter_mut().for_each(|s| *s *= scale);

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
}
