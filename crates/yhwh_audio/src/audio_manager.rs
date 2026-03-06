use std::{collections::HashMap, fs};

use kira::{AudioManagerSettings, DefaultBackend, PlaySoundError, Tween, sound::static_sound::StaticSoundData};

pub struct AudioManager {
    pub manager: kira::AudioManager,
    pub loaded_audios: HashMap<String, StaticSoundData>
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            manager: kira::AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).expect("AudioManager::new() error: Could not create Audio Manager!!"),
            loaded_audios: HashMap::new()
        }
    }

    pub fn load_audio(&mut self, file_name: &str) {
        self.loaded_audios.insert(file_name.to_string(), StaticSoundData::from_file(String::from("res/audio/") + &file_name).expect(&format!("Could not load audio file: {}", file_name)));
    }

    pub fn load_audios(&mut self, path: &str) {
        for file in fs::read_dir(path).expect(&format!("AudioManager::load_audios() error: cannot find path{} ", path)) {
            let file_path = file.unwrap().path();
            if let Some(file_name) = &file_path.file_name() {
                self.loaded_audios.insert(file_name.to_string_lossy().to_string(), StaticSoundData::from_file(&file_path).unwrap());

                println!("Loaded audio: {:?}", file_name);
            }
        }
    }

    pub fn play_audio(&mut self, audio_name: &str, play_rate: f64, volume: f32) -> Result<(), PlaySoundError<()>> {
        if let Some(sound_data) = self.loaded_audios.get(audio_name) {
           let mut sound = self.manager.play(sound_data.clone())?;

           sound.set_playback_rate(play_rate, Tween::default());
           sound.set_volume(volume, Tween::default());

           Ok(())
        } else {
            println!("Can not play audio: {}", audio_name);
            Err(PlaySoundError::SoundLimitReached)
        }
    }
}