use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VoiceAllocator {
    voices: Vec<char>,
    user_voices: HashMap<u64, char>,
    next_voice_index: usize,
}

impl VoiceAllocator {
    pub fn new(voices: Vec<char>) -> Self {
        Self {
            voices,
            user_voices: HashMap::new(),
            next_voice_index: 0,
        }
    }

    pub fn get_or_insert(&mut self, user_id: u64) -> char {
        if let Some(&voice) = self.user_voices.get(&user_id) {
            return voice;
        }

        let voice = self.voices[self.next_voice_index];
        self.user_voices.insert(user_id, voice);
        self.next_voice_index = (self.next_voice_index + 1) % self.voices.len();
        voice
    }

    pub fn remove(&mut self, user_id: u64) {
        self.user_voices.remove(&user_id);
    }

    pub fn get_users(&self) -> Vec<u64> {
        self.user_voices.keys().copied().collect()
    }
}
