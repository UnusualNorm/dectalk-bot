use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    io::Cursor,
    sync::Arc,
};

use dectalk::PAUL_VOICE;
use regex::Regex;
use serenity::{
    all::{GuildId, UserId, VoiceState},
    async_trait,
    client::{Client, Context, EventHandler},
    model::{channel::Message, gateway::Ready},
    prelude::{GatewayIntents, TypeMapKey},
};
use songbird::{input::Input, tracks::Track, SerenityInit};
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
    signal,
    sync::Mutex,
};
use voice_manager::VoiceManager;

mod dectalk;
mod voice_manager;

struct VoiceManagerKey;

impl TypeMapKey for VoiceManagerKey {
    type Value = Arc<VoiceManager>;
}

struct GuildUsersKey;

impl TypeMapKey for GuildUsersKey {
    type Value = Arc<Mutex<HashMap<GuildId, HashSet<UserId>>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        let author_id = new_message.author.id;
        let guild_id = match new_message.guild_id {
            Some(guild_id) => guild_id,
            None => {
                eprintln!("Failed to get guild id");
                return;
            }
        };

        let is_owner = author_id.get()
            == env::var("DISCORD_OWNER")
                .expect("Expected a owner in the environment")
                .parse::<u64>()
                .expect("Expected the owner to be a u64");

        let voice_manager = match ctx.data.read().await.get::<VoiceManagerKey>() {
            Some(voice_manager) => voice_manager.clone(),
            None => {
                eprintln!("Failed to get voice manager");
                return;
            }
        };

        let requested_roll = get_requested_roll(&new_message.content);
        if let Some(roll) = requested_roll {
            println!("Setting roll for {}: {}", author_id, roll);
            if let Err(e) = voice_manager.set_roll(author_id.get(), roll).await {
                eprintln!("Failed to set roll: {:?}", e);
                return;
            }
        }

        if !is_owner && new_message.content.len() > 256 {
            return;
        }

        let content = remove_requested_roll(&process_message(&new_message.content));
        if content.is_empty() {
            return;
        }

        let user_channel_id = {
            let guild = match new_message.guild(&ctx.cache) {
                Some(guild) => guild,
                None => {
                    eprintln!("Failed to get guild");
                    return;
                }
            };

            let voice_states = match guild.voice_states.get(&author_id) {
                Some(voice_states) => voice_states,
                None => {
                    eprintln!("Failed to get voice states");
                    return;
                }
            };

            match voice_states.channel_id {
                Some(channel_id) => channel_id,
                None => {
                    eprintln!("Failed to get channel id");
                    return;
                }
            }
        };

        let channel_id = new_message.channel_id;
        if user_channel_id != channel_id {
            return;
        }

        println!("Found valid message from {}", author_id);

        let manager = match songbird::get(&ctx).await {
            Some(manager) => manager,
            None => {
                eprintln!("Failed to get songbird manager");
                return;
            }
        };

        let handler_lock = manager.get_or_insert(guild_id);
        let mut handler = handler_lock.lock().await;

        if let Err(e) = handler.join(channel_id).await {
            eprintln!("Failed to join channel: {:?}", e);
            return;
        }

        let voice = voice_manager.get_voice(author_id.get()).await;
        let tts_path =
            match dectalk::tts(&content, if is_owner { &PAUL_VOICE } else { &voice }).await {
                Ok(tts_path) => tts_path,
                Err(e) => {
                    eprintln!("Failed to generate TTS: {:?}", e);
                    return;
                }
            };

        let mut tts_file = match File::open(&tts_path).await {
            Ok(tts_file) => tts_file,
            Err(e) => {
                eprintln!("Failed to open TTS file: {:?}", e);
                return;
            }
        };

        let mut tts_bytes = Vec::new();
        if let Err(e) = tts_file.read_to_end(&mut tts_bytes).await {
            eprintln!("Failed to read TTS file: {:?}", e);
            return;
        }

        if let Err(e) = fs::remove_file(&tts_path).await {
            eprintln!("Failed to remove TTS file: {:?}", e);
            return;
        }

        let duration = match get_wav_duration(&tts_bytes).await {
            Some(duration) => duration,
            None => {
                eprintln!("Failed to get duration");
                return;
            }
        };

        if !is_owner && duration > 15.0 {
            eprintln!("TTS duration is too long");
            return;
        }

        let guild_users = match ctx.data.read().await.get::<GuildUsersKey>() {
            Some(guild_users) => guild_users.clone(),
            None => {
                eprintln!("Failed to get guild users");
                return;
            }
        };

        let normalized_tts_bytes = match normalize_wav_volume(&tts_bytes) {
            Ok(normalized_tts_bytes) => normalized_tts_bytes,
            Err(e) => {
                eprintln!("Failed to normalize TTS volume: {:?}", e);
                return;
            }
        };

        let mut guild_users = guild_users.lock().await;
        guild_users
            .entry(guild_id)
            .or_insert_with(HashSet::new)
            .insert(author_id);

        handler.play(Track::from(Input::from(normalized_tts_bytes)).volume(0.25));
    }

    async fn voice_state_update(&self, ctx: Context, _old: Option<VoiceState>, new: VoiceState) {
        let guild_id = match new.guild_id {
            Some(guild_id) => guild_id,
            None => {
                eprintln!("Failed to get guild id");
                return;
            }
        };

        let guild_users = match ctx.data.read().await.get::<GuildUsersKey>() {
            Some(guild_users) => guild_users.clone(),
            None => {
                eprintln!("Failed to get guild users");
                return;
            }
        };
        let mut guild_users = guild_users.lock().await;

        if new.channel_id.is_none() {
            guild_users
                .entry(guild_id)
                .or_insert_with(HashSet::new)
                .remove(&new.user_id);
        }

        if guild_users
            .entry(guild_id)
            .or_insert_with(HashSet::new)
            .is_empty()
        {
            let manager = match songbird::get(&ctx).await {
                Some(manager) => manager,
                None => {
                    eprintln!("Failed to get songbird manager");
                    return;
                }
            };

            let handler_lock = match manager.get(guild_id) {
                Some(handler_lock) => handler_lock,
                None => {
                    eprintln!("Failed to get handler lock");
                    return;
                }
            };
            let mut handler = handler_lock.lock().await;

            if let Err(e) = handler.leave().await {
                println!("Failed to leave channel: {:?}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let voice_manager = VoiceManager::new();
    match voice_manager.load_rolls().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to load rolls: {:?}", e);
        }
    }

    let mut client = Client::builder(
        &env::var("DISCORD_TOKEN")?,
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
    .type_map_insert::<VoiceManagerKey>(Arc::new(voice_manager))
    .type_map_insert::<GuildUsersKey>(Arc::new(Mutex::new(HashMap::new())))
    .event_handler(Handler)
    .register_songbird()
    .await
    .expect("Err creating client");

    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|e| eprintln!("Client ended: {:?}", e));
    });

    let _signal_err = signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");
    Ok(())
}

async fn get_wav_duration(wav_bytes: &[u8]) -> Option<f64> {
    let mut cursor = Cursor::new(wav_bytes);

    let mut riff_header = [0; 12];
    cursor.read_exact(&mut riff_header).await.ok()?;

    if &riff_header[0..4] != b"RIFF" || &riff_header[8..12] != b"WAVE" {
        return None;
    }

    let mut fmt_chunk_header = [0; 8];
    cursor.read_exact(&mut fmt_chunk_header).await.ok()?;

    if &fmt_chunk_header[0..4] != b"fmt " {
        return None;
    }

    let fmt_chunk_size = u32::from_le_bytes(fmt_chunk_header[4..8].try_into().ok()?);

    let mut fmt_chunk_data = vec![0; fmt_chunk_size as usize];
    cursor.read_exact(&mut fmt_chunk_data).await.ok()?;

    let audio_format = u16::from_le_bytes(fmt_chunk_data[0..2].try_into().ok()?);
    // let num_channels = u16::from_le_bytes(fmt_chunk_data[2..4].try_into().ok()?);
    let sample_rate = u32::from_le_bytes(fmt_chunk_data[4..8].try_into().ok()?);
    // let byte_rate = u32::from_le_bytes(fmt_chunk_data[8..12].try_into().ok()?);
    let block_align = u16::from_le_bytes(fmt_chunk_data[12..14].try_into().ok()?);
    // let bits_per_sample = u16::from_le_bytes(fmt_chunk_data[14..16].try_into().ok()?);

    if audio_format != 1 {
        return None;
    }

    let mut data_chunk_header = [0; 8];
    cursor.read_exact(&mut data_chunk_header).await.ok()?;

    while &data_chunk_header[0..4] != b"data" {
        let chunk_size = u32::from_le_bytes(data_chunk_header[4..8].try_into().ok()?);
        cursor.set_position(cursor.position() + chunk_size as u64);
        cursor.read_exact(&mut data_chunk_header).await.ok()?;
    }

    let data_chunk_size = u32::from_le_bytes(data_chunk_header[4..8].try_into().ok()?);

    let num_samples = data_chunk_size as f64 / block_align as f64;
    let duration = num_samples / sample_rate as f64;

    Some(duration)
}

fn get_requested_roll(content: &str) -> Option<u64> {
    let re = Regex::new(r"\[:roll\s*(\d+)\s*\]").unwrap();
    let caps = re.captures(content)?;
    let roll = caps.get(1)?.as_str().parse::<u64>().ok()?;
    Some(roll)
}

fn remove_requested_roll(content: &str) -> String {
    let re = Regex::new(r"\[:roll\s*\d+\s*\]").unwrap();
    re.replace_all(content, "").to_string()
}

fn process_message(text: &str) -> String {
    let text = remove_links(text);
    let text = replace_discord_emojis(&text);
    text.trim().to_string()
}

fn remove_links(text: &str) -> String {
    let url_pattern = r"https?://[^\s/$.?#].[^\s]*";
    let re = Regex::new(url_pattern).unwrap();
    re.replace_all(text, "").to_string()
}

fn replace_discord_emojis(text: &str) -> String {
    let emoji_pattern = r"<a?:(\w+):\d+>";
    let re = Regex::new(emoji_pattern).unwrap();
    let result = re.replace_all(text, |caps: &regex::Captures| {
        let emoji_name = caps.get(1).unwrap().as_str().to_string();
        emoji_name
    });

    result.to_string()
}

fn normalize_wav_volume(wav_file: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut reader = hound::WavReader::new(Cursor::new(wav_file))?;
    let spec = reader.spec();
    let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap_or(0)).collect();
    let max_sample = samples.iter().cloned().fold(0, i16::max);
    let min_sample = samples.iter().cloned().fold(0, i16::min);
    let max_amplitude = i16::max_value();
    let min_amplitude = i16::min_value();
    let mut normalized_samples = Vec::with_capacity(samples.len());
    for sample in samples {
        let normalized_sample = if sample > 0 {
            sample as f64 / max_sample as f64 * max_amplitude as f64
        } else {
            sample as f64 / min_sample as f64 * min_amplitude as f64
        };
        normalized_samples.push(normalized_sample as i16);
    }
    let mut buf = Vec::new();
    let mut writer = hound::WavWriter::new(Cursor::new(&mut buf), spec)?;
    for sample in normalized_samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    Ok(buf)
}
