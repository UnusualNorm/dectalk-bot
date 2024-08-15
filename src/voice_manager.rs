use std::{collections::HashMap, error::Error, sync::Arc};

use crate::dectalk::DectalkVoice;
use tokio::{fs, sync::Mutex};

pub struct VoiceManager {
    pub voices: Arc<Mutex<HashMap<u64, DectalkVoice>>>,
    pub rolls: Arc<Mutex<HashMap<u64, u64>>>,
}

impl VoiceManager {
    pub fn new() -> Self {
        VoiceManager {
            voices: Arc::new(Mutex::new(HashMap::new())),
            rolls: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_voice(&self, id: u64) -> DectalkVoice {
        println!("Getting voice for {}", id);
        let mut voices = self.voices.lock().await;
        if let Some(voice) = voices.get(&id) {
            return voice.clone();
        }

        println!("Generating voice for {}", id);
        let rolls = self.rolls.lock().await;
        let roll = rolls.get(&id).unwrap_or(&0);

        let voice = DectalkVoice::generate(id, *roll);
        voices.insert(id, voice.clone());
        voice
    }

    pub async fn clear_voice(&self, id: u64) {
        println!("Clearing voice for {}", id);
        self.voices.lock().await.remove(&id);
    }

    pub async fn set_roll(&self, id: u64, roll: u64) -> Result<(), Box<dyn Error>> {
        println!("Setting roll for {}: {}", id, roll);
        self.rolls.lock().await.insert(id, roll);
        self.clear_voice(id).await;
        self.save_rolls().await?;
        Ok(())
    }

    pub async fn load_rolls(&self) -> Result<(), Box<dyn Error>> {
        println!("Loading rolls...");
        let rolls_string = fs::read_to_string("data/rolls.json").await?;
        let mut rolls = self.rolls.lock().await;
        *rolls = serde_json::from_str(&rolls_string)?;
        Ok(())
    }

    pub async fn save_rolls(&self) -> Result<(), Box<dyn Error>> {
        println!("Saving rolls...");
        let rolls = self.rolls.lock().await;
        let rolls_string = serde_json::to_string(&*rolls)?;
        fs::write("data/rolls.json", rolls_string).await?;
        Ok(())
    }
}
