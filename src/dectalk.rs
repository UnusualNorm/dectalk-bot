use std::error::Error;

use tiny_keccak::keccakf;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DectalkVoice {
    average_pitch: u16,                  // Hz
    assertiveness: u8,                   // %
    fourth_formant_bandwidth: u16,       // Hz
    fifth_formant_bandwidth: u16,        // Hz
    baseline_fall: u16,                  // Hz
    breathiness: u8,                     // dB
    fourth_formant_resonance: u16,       // Hz
    fifth_formant_resonance: u16,        // Hz
    hat_rise: u16,                       // Hz
    head_size: u8,                       // %
    laryngealization: u8,                // %
    lax_breathiness: u8,                 // %
    num_fixed_samples_open_glottis: u16, // samples
    pitch_range: u8,                     // %
    quickness: u8,                       // %
    richness: u8,                        // %
    smoothness: u8,                      // %
    stress_rise: u16,                    // Hz
    sex: u8,                             // 1 (male) or 0 (female)
}

pub const PAUL_VOICE: DectalkVoice = DectalkVoice {
    average_pitch: 112,
    assertiveness: 100,
    fourth_formant_bandwidth: 280,
    fifth_formant_bandwidth: 330,
    baseline_fall: 18,
    breathiness: 0,
    fourth_formant_resonance: 3300,
    fifth_formant_resonance: 3650,
    hat_rise: 18,
    head_size: 100,
    laryngealization: 0,
    lax_breathiness: 0,
    num_fixed_samples_open_glottis: 10,
    pitch_range: 100,
    quickness: 40,
    richness: 70,
    smoothness: 30,
    stress_rise: 25,
    sex: 1,
};

pub const WENDY_VOICE: DectalkVoice = DectalkVoice {
    average_pitch: 195,
    assertiveness: 55,
    fourth_formant_bandwidth: 300,
    fifth_formant_bandwidth: 2048,
    baseline_fall: 10,
    breathiness: 45,
    fourth_formant_resonance: 4600,
    fifth_formant_resonance: 2500,
    hat_rise: 18,
    head_size: 100,
    laryngealization: 0,
    lax_breathiness: 80,
    num_fixed_samples_open_glottis: 15,
    pitch_range: 100,
    quickness: 20,
    richness: 70,
    smoothness: 20,
    stress_rise: 22,
    sex: 1,
};

const fn const_u16_max(num1: u16, num2: u16) -> u16 {
    if num1 > num2 {
        num1
    } else {
        num2
    }
}
const fn const_u16_min(num1: u16, num2: u16) -> u16 {
    if num1 < num2 {
        num1
    } else {
        num2
    }
}
const fn const_u16_range(num1: u16, num2: u16) -> u16 {
    const_u16_max(num1, num2) - const_u16_min(num1, num2)
}
const fn const_u16_clamped_subtract(num1: u16, num2: u16) -> u16 {
    if num2 > num1 {
        0
    } else {
        num1 - num2
    }
}
const fn const_u16_clamped_add(num1: u16, num2: u16) -> u16 {
    if num2 > u16::MAX - num1 {
        u16::MAX
    } else {
        num1 + num2
    }
}

const fn const_u8_max(num1: u8, num2: u8) -> u8 {
    if num1 > num2 {
        num1
    } else {
        num2
    }
}
const fn const_u8_min(num1: u8, num2: u8) -> u8 {
    if num1 < num2 {
        num1
    } else {
        num2
    }
}
const fn const_u8_range(num1: u8, num2: u8) -> u8 {
    const_u8_max(num1, num2) - const_u8_min(num1, num2)
}
const fn const_u8_clamped_subtract(num1: u8, num2: u8) -> u8 {
    if num2 > num1 {
        0
    } else {
        num1 - num2
    }
}
const fn const_u8_clamped_add(num1: u8, num2: u8) -> u8 {
    if num2 > u8::MAX - num1 {
        u8::MAX
    } else {
        num1 + num2
    }
}

const AVERAGE_PITCH_RANGE: u16 =
    const_u16_range(PAUL_VOICE.average_pitch, WENDY_VOICE.average_pitch);
const AVERAGE_PITCH_MIN: u16 =
    const_u16_clamped_subtract(PAUL_VOICE.average_pitch, AVERAGE_PITCH_RANGE);
const AVERAGE_PITCH_MAX: u16 = const_u16_clamped_add(PAUL_VOICE.average_pitch, AVERAGE_PITCH_RANGE);
const ASSERTIVENESS_RANGE: u8 = const_u8_range(PAUL_VOICE.assertiveness, WENDY_VOICE.assertiveness);
const ASSERTIVENESS_MIN: u8 =
    const_u8_clamped_subtract(PAUL_VOICE.assertiveness, ASSERTIVENESS_RANGE);
const ASSERTIVENESS_MAX: u8 = const_u8_clamped_add(PAUL_VOICE.assertiveness, ASSERTIVENESS_RANGE);
const FOURTH_FORMANT_BANDWIDTH_RANGE: u16 = const_u16_range(
    PAUL_VOICE.fourth_formant_bandwidth,
    WENDY_VOICE.fourth_formant_bandwidth,
);
const FOURTH_FORMANT_BANDWIDTH_MIN: u16 = const_u16_clamped_subtract(
    PAUL_VOICE.fourth_formant_bandwidth,
    FOURTH_FORMANT_BANDWIDTH_RANGE,
);
const FOURTH_FORMANT_BANDWIDTH_MAX: u16 = const_u16_clamped_add(
    PAUL_VOICE.fourth_formant_bandwidth,
    FOURTH_FORMANT_BANDWIDTH_RANGE,
);
const FIFTH_FORMANT_BANDWIDTH_RANGE: u16 = const_u16_range(
    PAUL_VOICE.fifth_formant_bandwidth,
    WENDY_VOICE.fifth_formant_bandwidth,
);
const FIFTH_FORMANT_BANDWIDTH_MIN: u16 = const_u16_clamped_subtract(
    PAUL_VOICE.fifth_formant_bandwidth,
    FIFTH_FORMANT_BANDWIDTH_RANGE,
);
const FIFTH_FORMANT_BANDWIDTH_MAX: u16 = const_u16_clamped_add(
    PAUL_VOICE.fifth_formant_bandwidth,
    FIFTH_FORMANT_BANDWIDTH_RANGE,
);
const BASELINE_FALL_RANGE: u16 =
    const_u16_range(PAUL_VOICE.baseline_fall, WENDY_VOICE.baseline_fall);
const BASELINE_FALL_MIN: u16 =
    const_u16_clamped_subtract(PAUL_VOICE.baseline_fall, BASELINE_FALL_RANGE);
const BASELINE_FALL_MAX: u16 = const_u16_clamped_add(PAUL_VOICE.baseline_fall, BASELINE_FALL_RANGE);
const BREATHINESS_RANGE: u8 = const_u8_range(PAUL_VOICE.breathiness, WENDY_VOICE.breathiness);
const BREATHINESS_MIN: u8 = const_u8_clamped_subtract(PAUL_VOICE.breathiness, BREATHINESS_RANGE);
const BREATHINESS_MAX: u8 = const_u8_clamped_add(PAUL_VOICE.breathiness, BREATHINESS_RANGE);
const FOURTH_FORMANT_RESONANCE_RANGE: u16 = const_u16_range(
    PAUL_VOICE.fourth_formant_resonance,
    WENDY_VOICE.fourth_formant_resonance,
);
const FOURTH_FORMANT_RESONANCE_MIN: u16 = const_u16_clamped_subtract(
    PAUL_VOICE.fourth_formant_resonance,
    FOURTH_FORMANT_RESONANCE_RANGE,
);
const FOURTH_FORMANT_RESONANCE_MAX: u16 = const_u16_clamped_add(
    PAUL_VOICE.fourth_formant_resonance,
    FOURTH_FORMANT_RESONANCE_RANGE,
);
const FIFTH_FORMANT_RESONANCE_RANGE: u16 = const_u16_range(
    PAUL_VOICE.fifth_formant_resonance,
    WENDY_VOICE.fifth_formant_resonance,
);
const FIFTH_FORMANT_RESONANCE_MIN: u16 = const_u16_clamped_subtract(
    PAUL_VOICE.fifth_formant_resonance,
    FIFTH_FORMANT_RESONANCE_RANGE,
);
const FIFTH_FORMANT_RESONANCE_MAX: u16 = const_u16_clamped_add(
    PAUL_VOICE.fifth_formant_resonance,
    FIFTH_FORMANT_RESONANCE_RANGE,
);
const HAT_RISE_RANGE: u16 = const_u16_range(PAUL_VOICE.hat_rise, WENDY_VOICE.hat_rise);
const HAT_RISE_MIN: u16 = const_u16_clamped_subtract(PAUL_VOICE.hat_rise, HAT_RISE_RANGE);
const HAT_RISE_MAX: u16 = const_u16_clamped_add(PAUL_VOICE.hat_rise, HAT_RISE_RANGE);
const HEAD_SIZE_RANGE: u8 = const_u8_range(PAUL_VOICE.head_size, WENDY_VOICE.head_size);
const HEAD_SIZE_MIN: u8 = const_u8_clamped_subtract(PAUL_VOICE.head_size, HEAD_SIZE_RANGE);
const HEAD_SIZE_MAX: u8 = const_u8_clamped_add(PAUL_VOICE.head_size, HEAD_SIZE_RANGE);
const LARYNGEALIZATION_RANGE: u8 =
    const_u8_range(PAUL_VOICE.laryngealization, WENDY_VOICE.laryngealization);
const LARYNGEALIZATION_MIN: u8 =
    const_u8_clamped_subtract(PAUL_VOICE.laryngealization, LARYNGEALIZATION_RANGE);
const LARYNGEALIZATION_MAX: u8 =
    const_u8_clamped_add(PAUL_VOICE.laryngealization, LARYNGEALIZATION_RANGE);
const LAX_BREATHINESS_RANGE: u8 =
    const_u8_range(PAUL_VOICE.lax_breathiness, WENDY_VOICE.lax_breathiness);
const LAX_BREATHINESS_MIN: u8 =
    const_u8_clamped_subtract(PAUL_VOICE.lax_breathiness, LAX_BREATHINESS_RANGE);
const LAX_BREATHINESS_MAX: u8 =
    const_u8_clamped_add(PAUL_VOICE.lax_breathiness, LAX_BREATHINESS_RANGE);
const NUM_FIXED_SAMPLES_OPEN_GLOTTIS_RANGE: u16 = const_u16_range(
    PAUL_VOICE.num_fixed_samples_open_glottis,
    WENDY_VOICE.num_fixed_samples_open_glottis,
);
const NUM_FIXED_SAMPLES_OPEN_GLOTTIS_MIN: u16 = const_u16_clamped_subtract(
    PAUL_VOICE.num_fixed_samples_open_glottis,
    NUM_FIXED_SAMPLES_OPEN_GLOTTIS_RANGE,
);
const NUM_FIXED_SAMPLES_OPEN_GLOTTIS_MAX: u16 = const_u16_clamped_add(
    PAUL_VOICE.num_fixed_samples_open_glottis,
    NUM_FIXED_SAMPLES_OPEN_GLOTTIS_RANGE,
);
const PITCH_RANGE_RANGE: u8 = const_u8_range(PAUL_VOICE.pitch_range, WENDY_VOICE.pitch_range);
const PITCH_RANGE_MIN: u8 = const_u8_clamped_subtract(PAUL_VOICE.pitch_range, PITCH_RANGE_RANGE);
const PITCH_RANGE_MAX: u8 = const_u8_clamped_add(PAUL_VOICE.pitch_range, PITCH_RANGE_RANGE);
const QUICKNESS_RANGE: u8 = const_u8_range(PAUL_VOICE.quickness, WENDY_VOICE.quickness);
const QUICKNESS_MIN: u8 = const_u8_clamped_subtract(PAUL_VOICE.quickness, QUICKNESS_RANGE);
const QUICKNESS_MAX: u8 = const_u8_clamped_add(PAUL_VOICE.quickness, QUICKNESS_RANGE);
const RICHNESS_RANGE: u8 = const_u8_range(PAUL_VOICE.richness, WENDY_VOICE.richness);
const RICHNESS_MIN: u8 = const_u8_clamped_subtract(PAUL_VOICE.richness, RICHNESS_RANGE);
const RICHNESS_MAX: u8 = const_u8_clamped_add(PAUL_VOICE.richness, RICHNESS_RANGE);
const SMOOTHNESS_RANGE: u8 = const_u8_range(PAUL_VOICE.smoothness, WENDY_VOICE.smoothness);
const SMOOTHNESS_MIN: u8 = const_u8_clamped_subtract(PAUL_VOICE.smoothness, SMOOTHNESS_RANGE);
const SMOOTHNESS_MAX: u8 = const_u8_clamped_add(PAUL_VOICE.smoothness, SMOOTHNESS_RANGE);
const STRESS_RISE_RANGE: u16 = const_u16_range(PAUL_VOICE.stress_rise, WENDY_VOICE.stress_rise);
const STRESS_RISE_MIN: u16 = const_u16_clamped_subtract(PAUL_VOICE.stress_rise, STRESS_RISE_RANGE);
const STRESS_RISE_MAX: u16 = const_u16_clamped_add(PAUL_VOICE.stress_rise, STRESS_RISE_RANGE);
const SEX_RANGE: u8 = const_u8_range(PAUL_VOICE.sex, WENDY_VOICE.sex);
const SEX_MIN: u8 = const_u8_clamped_subtract(PAUL_VOICE.sex, SEX_RANGE);
const SEX_MAX: u8 = const_u8_clamped_add(PAUL_VOICE.sex, SEX_RANGE);

#[inline]
const fn u64_to_range(min: u64, max: u64, value: u64) -> u64 {
    min + (value % (max - min + 1))
}

impl DectalkVoice {
    pub fn generate(player_id: u64, seed: u64) -> Self {
        let mut seed = [player_id ^ seed; 25];
        keccakf(&mut seed);
        let average_pitch =
            u64_to_range(AVERAGE_PITCH_MIN as u64, AVERAGE_PITCH_MAX as u64, seed[0]) as u16;
        keccakf(&mut seed);
        let assertiveness =
            u64_to_range(ASSERTIVENESS_MIN as u64, ASSERTIVENESS_MAX as u64, seed[0]) as u8;
        keccakf(&mut seed);
        let fourth_formant_bandwidth = u64_to_range(
            FOURTH_FORMANT_BANDWIDTH_MIN as u64,
            FOURTH_FORMANT_BANDWIDTH_MAX as u64,
            seed[0],
        ) as u16;
        keccakf(&mut seed);
        let fifth_formant_bandwidth = u64_to_range(
            FIFTH_FORMANT_BANDWIDTH_MIN as u64,
            FIFTH_FORMANT_BANDWIDTH_MAX as u64,
            seed[0],
        ) as u16;
        keccakf(&mut seed);
        let baseline_fall =
            u64_to_range(BASELINE_FALL_MIN as u64, BASELINE_FALL_MAX as u64, seed[0]) as u16;
        keccakf(&mut seed);
        let breathiness =
            u64_to_range(BREATHINESS_MIN as u64, BREATHINESS_MAX as u64, seed[0]) as u8;
        keccakf(&mut seed);
        let fourth_formant_resonance = u64_to_range(
            FOURTH_FORMANT_RESONANCE_MIN as u64,
            FOURTH_FORMANT_RESONANCE_MAX as u64,
            seed[0],
        ) as u16;
        keccakf(&mut seed);
        let fifth_formant_resonance = u64_to_range(
            FIFTH_FORMANT_RESONANCE_MIN as u64,
            FIFTH_FORMANT_RESONANCE_MAX as u64,
            seed[0],
        ) as u16;
        keccakf(&mut seed);
        let hat_rise = u64_to_range(HAT_RISE_MIN as u64, HAT_RISE_MAX as u64, seed[0]) as u16;
        keccakf(&mut seed);
        let head_size = u64_to_range(HEAD_SIZE_MIN as u64, HEAD_SIZE_MAX as u64, seed[0]) as u8;
        keccakf(&mut seed);
        let laryngealization = u64_to_range(
            LARYNGEALIZATION_MIN as u64,
            LARYNGEALIZATION_MAX as u64,
            seed[0],
        ) as u8;
        keccakf(&mut seed);
        let lax_breathiness = u64_to_range(
            LAX_BREATHINESS_MIN as u64,
            LAX_BREATHINESS_MAX as u64,
            seed[0],
        ) as u8;
        keccakf(&mut seed);
        let num_fixed_samples_open_glottis = u64_to_range(
            NUM_FIXED_SAMPLES_OPEN_GLOTTIS_MIN as u64,
            NUM_FIXED_SAMPLES_OPEN_GLOTTIS_MAX as u64,
            seed[0],
        ) as u16;
        keccakf(&mut seed);
        let pitch_range =
            u64_to_range(PITCH_RANGE_MIN as u64, PITCH_RANGE_MAX as u64, seed[0]) as u8;
        keccakf(&mut seed);
        let quickness = u64_to_range(QUICKNESS_MIN as u64, QUICKNESS_MAX as u64, seed[0]) as u8;
        keccakf(&mut seed);
        let richness = u64_to_range(RICHNESS_MIN as u64, RICHNESS_MAX as u64, seed[0]) as u8;
        keccakf(&mut seed);
        let smoothness = u64_to_range(SMOOTHNESS_MIN as u64, SMOOTHNESS_MAX as u64, seed[0]) as u8;
        keccakf(&mut seed);
        let stress_rise =
            u64_to_range(STRESS_RISE_MIN as u64, STRESS_RISE_MAX as u64, seed[0]) as u16;
        keccakf(&mut seed);
        let sex = u64_to_range(SEX_MIN as u64, SEX_MAX as u64, seed[0]) as u8;

        Self {
            average_pitch,
            assertiveness,
            fourth_formant_bandwidth,
            fifth_formant_bandwidth,
            baseline_fall,
            breathiness,
            fourth_formant_resonance,
            fifth_formant_resonance,
            hat_rise,
            head_size,
            laryngealization,
            lax_breathiness,
            num_fixed_samples_open_glottis,
            pitch_range,
            quickness,
            richness,
            smoothness,
            stress_rise,
            sex,
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
        [:dv ap {}][:dv as {}]
        [:dv b4 {}][:dv b5 {}]
        [:dv bf {}][:dv br {}]
        [:dv f4 {}][:dv f5 {}]
        [:dv hr {}][:dv hs {}]
        [:dv la {}][:dv lx {}]
        [:dv nf {}][:dv pr {}]
        [:dv qu {}][:dv ri {}]
        [:dv sm {}][:dv sr {}]
        [:dv sx {}]",
        voice.average_pitch,
        voice.assertiveness,
        voice.fourth_formant_bandwidth,
        voice.fifth_formant_bandwidth,
        voice.baseline_fall,
        voice.breathiness,
        voice.fourth_formant_resonance,
        voice.fifth_formant_resonance,
        voice.hat_rise,
        voice.head_size,
        voice.laryngealization,
        voice.lax_breathiness,
        voice.num_fixed_samples_open_glottis,
        voice.pitch_range,
        voice.quickness,
        voice.richness,
        voice.smoothness,
        voice.stress_rise,
        voice.sex
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
