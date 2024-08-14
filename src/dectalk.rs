use std::error::Error;

use tokio::process::Command;
use uuid::Uuid;

pub async fn tts(text: &str, voice: &char) -> Result<String, Box<dyn Error>> {
    let filename = format!("dectalk/{}.wav", Uuid::new_v4());

    let mut cmd = Command::new("dectalk/say");
    cmd.arg("-a").arg(text);
    cmd.arg("-fo").arg(&filename);
    cmd.arg("-pre").arg(format!("[:phoneme on][:n{}]", voice));

    let output = cmd.output().await?;
    if !output.status.success() {
        return Err(format!(
            "Failed to run say: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(filename)
}
