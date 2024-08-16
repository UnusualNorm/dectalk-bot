use std::error::Error;

use tiny_keccak::keccakf;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DectalkVoice {
    sx: u8,   // --     Set sex to female (0) or male (1)
    hs: u16,  // %      Head size
    f4: u16,  // Hz     Fourth formant frequency
    f5: u16,  // Hz     Fifth formant frequency
    b4: u16,  // Hz     Fourth formant bandwidth
    b5: u16,  // Hz     Fifth formant bandwidth
    br: u16,  // dB     Breathiness
    lx: u16,  // %      Lax breathiness
    sm: u16,  // %      Smoothness (high frequency attenuation)
    ri: u16,  // %      Richness
    nf: u16,  // --     Number of fixed samplings of glottal pulse open phase
    la: u16,  // %      Laryngealization
    bf: u16,  // Hz     Baseline fall
    hr: u16,  // Hz     Hat rise
    sr: u16,  // Hz     Stress rise
    as_: u16, // %      Assertiveness
    qu: u16,  // %      Quickness
    ap: u16,  // Hz     Average pitch
    pr: u16,  // %      Pitch range
              // gv: u16,  // dB     Gain of voicing source
              // gh: u16,  // dB     Gain of aspiration source
              // gn: u16,  // dB     Gain of frication source
              // gf: u16,  // bB     Gain of nasalization
              // g1: u16,  // dB     Gain of first formant resonator
              // g2: u16,  // dB     Gain of second formant resonator
              // g3: u16,  // dB     Gain of third formant resonator
              // g4: u16,  // dB     Gain of fourth formant resonator
              // g5: u16,  // dB     Gain of fifth formant resonator (replaces lo)
}

pub const PAUL_VOICE: DectalkVoice = DectalkVoice {
    sx: 1,
    hs: 100,
    f4: 3300,
    f5: 3650,
    b4: 260,
    b5: 330,
    br: 0,
    lx: 0,
    sm: 3,
    ri: 70,
    nf: 0,
    la: 0,
    bf: 18,
    hr: 18,
    sr: 32,
    as_: 100,
    qu: 40,
    ap: 112,
    pr: 100,
    // gv: 65,
    // gh: 70,
    // gn: 74,
    // gf: 70,
    // g1: 68,
    // g2: 60,
    // g3: 48,
    // g4: 64,
    // g5: 86,
};

#[inline]
const fn u64_to_u16_loop(min: u16, max: u16, value: u64) -> u16 {
    (min as u64 + (value % (max - min + 1) as u64)) as u16
}

impl DectalkVoice {
    pub fn generate(player_id: u64, seed: u64) -> Self {
        let mut random = [player_id ^ seed; 25];
        let sx = (seed % 2) as u8;
        keccakf(&mut random);
        let hs = u64_to_u16_loop(65, 145, random[0]);
        keccakf(&mut random);
        let f4 = u64_to_u16_loop(2000, 4650, random[0]);
        keccakf(&mut random);
        let f5 = u64_to_u16_loop(2500, 4950, random[0]);
        keccakf(&mut random);
        let b4 = u64_to_u16_loop(100, 2048, random[0]);
        keccakf(&mut random);
        let b5 = u64_to_u16_loop(100, 2048, random[0]);
        keccakf(&mut random);
        let br = u64_to_u16_loop(0, 72, random[0]);
        keccakf(&mut random);
        let lx = u64_to_u16_loop(0, 100, random[0]);
        keccakf(&mut random);
        let sm = u64_to_u16_loop(0, 100, random[0]);
        keccakf(&mut random);
        let ri = u64_to_u16_loop(0, 100, random[0]);
        keccakf(&mut random);
        let nf = u64_to_u16_loop(0, 100, random[0]);
        keccakf(&mut random);
        let la = u64_to_u16_loop(0, 100, random[0]);
        keccakf(&mut random);
        let bf = u64_to_u16_loop(0, 40, random[0]);
        keccakf(&mut random);
        let hr = u64_to_u16_loop(2, 100, random[0]);
        keccakf(&mut random);
        let sr = u64_to_u16_loop(1, 100, random[0]);
        keccakf(&mut random);
        let as_ = u64_to_u16_loop(0, 100, random[0]);
        keccakf(&mut random);
        let qu = u64_to_u16_loop(0, 100, random[0]);
        keccakf(&mut random);
        let ap = u64_to_u16_loop(50, 350, random[0]);
        keccakf(&mut random);
        let pr = u64_to_u16_loop(0, 250, random[0]);
        // keccakf(&mut random);
        // let gv = u64_to_u16_loop(0, 86, random[0]);
        // keccakf(&mut random);
        // let gh = u64_to_u16_loop(0, 86, random[0]);
        // keccakf(&mut random);
        // let gn = u64_to_u16_loop(0, 86, random[0]);
        // keccakf(&mut random);
        // let gf = u64_to_u16_loop(0, 86, random[0]);
        // keccakf(&mut random);
        // let g1 = u64_to_u16_loop(0, 86, random[0]);
        // keccakf(&mut random);
        // let g2 = u64_to_u16_loop(0, 86, random[0]);
        // keccakf(&mut random);
        // let g3 = u64_to_u16_loop(0, 86, random[0]);
        // keccakf(&mut random);
        // let g4 = u64_to_u16_loop(0, 86, random[0]);
        // keccakf(&mut random);
        // let g5 = u64_to_u16_loop(0, 86, random[0]);

        Self {
            sx,
            hs,
            f4,
            f5,
            b4,
            b5,
            br,
            lx,
            sm,
            ri,
            nf,
            la,
            bf,
            hr,
            sr,
            as_,
            qu,
            ap,
            pr,
            // gv,
            // gh,
            // gn,
            // gf,
            // g1,
            // g2,
            // g3,
            // g4,
            // g5,
        }
    }
}

pub async fn tts(text: &str, voice: &DectalkVoice) -> Result<String, Box<dyn Error>> {
    let filename = format!("dectalk/{}.wav", Uuid::new_v4());

    let mut cmd = Command::new("dectalk/say");
    cmd.arg("-a").arg(text);
    cmd.arg("-fo").arg(&filename);
    cmd.arg("-pre").arg(format!(
        "[:phoneme on][:nv]
        [:dv sx {}][:dv hs {}]
        [:dv f4 {}][:dv f5 {}]
        [:dv b4 {}][:dv b5 {}]
        [:dv br {}][:dv lx {}]
        [:dv sm {}][:dv ri {}]
        [:dv nf {}][:dv la {}]
        [:dv bf {}][:dv hr {}]
        [:dv sr {}][:dv as {}]
        [:dv qu {}][:dv ap {}]
        [:dv pr {}]", // [:dv gv {}]
        // [:dv gh {}][:dv gn {}]
        // [:dv gf {}][:dv g1 {}]
        // [:dv g2 {}][:dv g3 {}]
        // [:dv g4 {}][:dv g5 {}]",
        voice.sx,
        voice.hs,
        voice.f4,
        voice.f5,
        voice.b4,
        voice.b5,
        voice.br,
        voice.lx,
        voice.sm,
        voice.ri,
        voice.nf,
        voice.la,
        voice.bf,
        voice.hr,
        voice.sr,
        voice.as_,
        voice.qu,
        voice.ap,
        voice.pr,
        // voice.gv,
        // voice.gh,
        // voice.gn,
        // voice.gf,
        // voice.g1,
        // voice.g2,
        // voice.g3,
        // voice.g4,
        // voice.g5
    ));

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
