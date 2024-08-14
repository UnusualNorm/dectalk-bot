use std::{collections::HashMap, env, io::Cursor, sync::Arc};

use regex::Regex;
use serenity::{
    all::{GuildId, VoiceState},
    async_trait,
    client::{Client, Context, EventHandler},
    model::{channel::Message, gateway::Ready},
    prelude::{GatewayIntents, TypeMapKey},
};
use songbird::{input::Input, SerenityInit};
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
    signal,
    sync::Mutex,
};
use voice_allocator::VoiceAllocator;

mod dectalk;
mod voice_allocator;

struct VoiceAllocatorKey;

impl TypeMapKey for VoiceAllocatorKey {
    type Value = Arc<Mutex<HashMap<GuildId, VoiceAllocator>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        let is_owner = new_message.author.id.get()
            == env::var("DISCORD_OWNER")
                .expect("Expected a owner in the environment")
                .parse::<u64>()
                .expect("Expected the owner to be a u64");

        if !is_owner && new_message.content.len() > 128 {
            return;
        }

        let guild_id = match new_message.guild_id {
            Some(guild_id) => guild_id,
            None => {
                eprintln!("Failed to get guild id");
                return;
            }
        };

        let user_channel_id = {
            let guild = match new_message.guild(&ctx.cache) {
                Some(guild) => guild,
                None => {
                    eprintln!("Failed to get guild");
                    return;
                }
            };

            let voice_states = match guild.voice_states.get(&new_message.author.id) {
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

        let manager = match songbird::get(&ctx).await {
            Some(manager) => manager,
            None => {
                eprintln!("Failed to get songbird manager");
                return;
            }
        };

        let voice_allocator = match ctx.data.read().await.get::<VoiceAllocatorKey>() {
            Some(voice_allocator) => voice_allocator.clone(),
            None => {
                eprintln!("Failed to get voice allocator");
                return;
            }
        };
        let mut voice_allocators = voice_allocator.lock().await;

        let voice_allocator = match voice_allocators.get_mut(&guild_id) {
            Some(voice_allocator) => voice_allocator,
            None => {
                let voice_allocator =
                    VoiceAllocator::new(vec!['p', 'b', 'h', 'u', 'f', 'w', 'd', 'r', 'k']);
                voice_allocators.insert(guild_id, voice_allocator);
                voice_allocators.get_mut(&guild_id).unwrap()
            }
        };

        let handler_lock = manager.get_or_insert(guild_id);
        let mut handler = handler_lock.lock().await;

        if let Err(e) = handler.join(channel_id).await {
            eprintln!("Failed to join channel: {:?}", e);
            return;
        }

        let voice = if is_owner {
            &'p'
        } else {
            &voice_allocator.get_or_insert(new_message.author.id.get())
        };

        let tts_path = match dectalk::tts(&process_message(&new_message.content), voice).await {
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

        handler.play_input(Input::from(tts_bytes));
    }

    async fn voice_state_update(&self, ctx: Context, _old: Option<VoiceState>, new: VoiceState) {
        let guild_id = match new.guild_id {
            Some(guild_id) => guild_id,
            None => {
                eprintln!("Failed to get guild id");
                return;
            }
        };

        let voice_allocators = match ctx.data.read().await.get::<VoiceAllocatorKey>() {
            Some(voice_allocator) => voice_allocator.clone(),
            None => {
                eprintln!("Failed to get voice allocator");
                return;
            }
        };
        let mut voice_allocators = voice_allocators.lock().await;

        let voice_allocator = match voice_allocators.get_mut(&guild_id) {
            Some(voice_allocator) => voice_allocator,
            None => {
                eprintln!("Failed to get voice allocator");
                return;
            }
        };

        if new.channel_id.is_none() {
            voice_allocator.remove(new.user_id.get());
        }

        if voice_allocator.get_users().is_empty() {
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
async fn main() {
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .type_map_insert::<VoiceAllocatorKey>(Arc::new(Mutex::new(HashMap::new())))
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

fn process_message(text: &str) -> String {
    let text = remove_links(text);
    let text = replace_discord_emojis(&text);
    text
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
