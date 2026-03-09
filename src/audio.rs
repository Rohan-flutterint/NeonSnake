use macroquad::audio::{
    PlaySoundParams, Sound, load_sound_from_bytes, play_sound, set_sound_volume,
};

use crate::game::{AudioCue, LevelTheme};

pub struct SoundBank {
    music: Option<Sound>,
    key: Option<Sound>,
    eat: Option<Sound>,
    power_up: Option<Sound>,
    boom: Option<Sound>,
    game_over: Option<Sound>,
    current_theme: LevelTheme,
}

impl SoundBank {
    pub async fn load() -> Self {
        Self {
            music: load_generated_sound(generate_music_loop()).await,
            key: load_generated_sound(generate_key_sound()).await,
            eat: load_generated_sound(generate_eat_sound()).await,
            power_up: load_generated_sound(generate_power_up_sound()).await,
            boom: load_generated_sound(generate_boom_sound()).await,
            game_over: load_generated_sound(generate_game_over_sound()).await,
            current_theme: LevelTheme::Afterglow,
        }
    }

    pub fn start_music(&mut self, theme: LevelTheme) {
        self.current_theme = theme;
        if let Some(sound) = &self.music {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    volume: self.current_theme.music_volume(),
                },
            );
        }
    }

    pub fn apply_theme(&mut self, theme: LevelTheme) {
        if self.current_theme == theme {
            return;
        }

        self.current_theme = theme;
        if let Some(sound) = &self.music {
            set_sound_volume(sound, self.current_theme.music_volume());
        }
    }

    pub fn play(&self, cue: AudioCue) {
        let gain = self.current_theme.sfx_gain();
        match cue {
            AudioCue::Key => {
                if let Some(sound) = &self.key {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: false,
                            volume: (0.30 * gain).min(1.0),
                        },
                    );
                }
            }
            AudioCue::Eat => {
                if let Some(sound) = &self.eat {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: false,
                            volume: (0.45 * gain).min(1.0),
                        },
                    );
                }
            }
            AudioCue::PowerUp => {
                if let Some(sound) = &self.power_up {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: false,
                            volume: (0.48 * gain).min(1.0),
                        },
                    );
                }
            }
            AudioCue::Boom => {
                if let Some(sound) = &self.boom {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: false,
                            volume: (0.72 * gain).min(1.0),
                        },
                    );
                }
            }
            AudioCue::GameOver => {
                if let Some(sound) = &self.game_over {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: false,
                            volume: (0.60 * gain).min(1.0),
                        },
                    );
                }
            }
        }
    }
}

async fn load_generated_sound(data: Vec<u8>) -> Option<Sound> {
    load_sound_from_bytes(&data).await.ok()
}

fn generate_key_sound() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    let duration = 0.05;
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);

    for index in 0..total {
        let t = index as f32 / SAMPLE_RATE as f32;
        let progress = index as f32 / total as f32;
        let envelope = adsr(progress, 0.003, 0.0, 1.0, 0.55);
        let body = (std::f32::consts::TAU * 1320.0 * t).sin();
        let tail = (std::f32::consts::TAU * 980.0 * t).sin() * 0.35;
        samples.push((body * 0.8 + tail) * envelope * 0.28);
    }

    write_wav(&samples, SAMPLE_RATE)
}

fn generate_eat_sound() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    let duration = 0.11;
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);

    for index in 0..total {
        let t = index as f32 / SAMPLE_RATE as f32;
        let progress = index as f32 / total as f32;
        let freq = 700.0 + 520.0 * progress;
        let envelope = adsr(progress, 0.02, 0.0, 1.0, 0.22);
        let tone = (std::f32::consts::TAU * freq * t).sin();
        let sparkle = (std::f32::consts::TAU * (freq * 1.9) * t).sin() * 0.25;
        samples.push((tone * 0.9 + sparkle) * envelope * 0.42);
    }

    write_wav(&samples, SAMPLE_RATE)
}

fn generate_power_up_sound() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    let duration = 0.18;
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);

    for index in 0..total {
        let t = index as f32 / SAMPLE_RATE as f32;
        let progress = index as f32 / total as f32;
        let envelope = adsr(progress, 0.01, 0.03, 0.85, 0.40);
        let base = (std::f32::consts::TAU * (640.0 + progress * 260.0) * t).sin();
        let octave = (std::f32::consts::TAU * (1280.0 + progress * 320.0) * t).sin() * 0.35;
        let shimmer = (std::f32::consts::TAU * 12.0 * t).sin() * 0.08;
        samples.push((base * 0.9 + octave + shimmer) * envelope * 0.38);
    }

    write_wav(&samples, SAMPLE_RATE)
}

fn generate_boom_sound() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    let duration = 0.48;
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);

    for index in 0..total {
        let t = index as f32 / SAMPLE_RATE as f32;
        let progress = index as f32 / total as f32;
        let envelope = adsr(progress, 0.002, 0.03, 0.70, 0.78);
        let noise = (macroquad::rand::gen_range(-1000, 1000) as f32) / 1000.0;
        let rumble = (std::f32::consts::TAU * (86.0 - progress * 20.0) * t).sin() * 0.65;
        let crack = (std::f32::consts::TAU * 240.0 * t).sin() * (1.0 - progress) * 0.2;
        samples.push((noise * 0.55 + rumble + crack) * envelope * 0.52);
    }

    write_wav(&samples, SAMPLE_RATE)
}

fn generate_music_loop() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    let duration = 6.0;
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);
    let progression = [
        [196.00, 246.94, 293.66],
        [220.00, 261.63, 329.63],
        [174.61, 220.00, 261.63],
        [196.00, 246.94, 329.63],
    ];
    let chord_len = SAMPLE_RATE as usize + SAMPLE_RATE as usize / 2;

    for index in 0..total {
        let t = index as f32 / SAMPLE_RATE as f32;
        let chord_index = (index / chord_len) % progression.len();
        let chord = progression[chord_index];
        let chord_progress = (index % chord_len) as f32 / chord_len as f32;

        let pad = chord
            .iter()
            .enumerate()
            .map(|(voice, freq)| {
                let detune = 1.0 + voice as f32 * 0.003;
                (std::f32::consts::TAU * freq * detune * t).sin() * (0.26 - voice as f32 * 0.04)
            })
            .sum::<f32>();

        let arp_step = ((t * 4.0) as usize) % chord.len();
        let arp_freq = chord[arp_step] * 2.0;
        let arp = (std::f32::consts::TAU * arp_freq * t).sin().max(0.0) * 0.12;

        let sweep = (std::f32::consts::TAU * 0.18 * t).sin() * 0.08;
        let fade = (0.82 - (chord_progress - 0.5).abs() * 0.18).clamp(0.72, 0.88);
        samples.push((pad + arp + sweep) * fade * 0.34);
    }

    write_wav(&samples, SAMPLE_RATE)
}

fn generate_game_over_sound() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    let duration = 0.38;
    let total = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(total);

    for index in 0..total {
        let t = index as f32 / SAMPLE_RATE as f32;
        let progress = index as f32 / total as f32;
        let freq = 340.0 - 220.0 * progress;
        let envelope = adsr(progress, 0.01, 0.0, 1.0, 0.70);
        let wobble = (std::f32::consts::TAU * 7.0 * t).sin() * 8.0;
        let base = (std::f32::consts::TAU * (freq + wobble) * t).sin();
        let undertone = (std::f32::consts::TAU * (freq * 0.48) * t).sin() * 0.45;
        samples.push((base * 0.8 + undertone) * envelope * 0.5);
    }

    write_wav(&samples, SAMPLE_RATE)
}

fn adsr(progress: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> f32 {
    if progress < attack {
        return progress / attack.max(f32::EPSILON);
    }

    if progress > 1.0 - release {
        return ((1.0 - progress) / release.max(f32::EPSILON)).clamp(0.0, 1.0) * sustain;
    }

    if decay > 0.0 {
        let decay_end = attack + decay;
        if progress < decay_end {
            let t = (progress - attack) / decay;
            return 1.0 + (sustain - 1.0) * t;
        }
    }

    sustain
}

fn write_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_size = (samples.len() * block_align as usize) as u32;
    let chunk_size = 36 + data_size;

    let mut bytes = Vec::with_capacity((44 + data_size) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&chunk_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());

    for sample in samples {
        let pcm = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }

    bytes
}
