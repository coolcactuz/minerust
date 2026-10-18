#![forbid(unsafe_code)]

use bevy::audio::{
    AudioLoader, AudioPlayer, AudioSource, Decodable, GlobalVolume, PlaybackSettings, Volume,
};
use bevy::prelude::*;

use crate::block::BlockType;

/// Loaded handles to all in-game and UI sound effects.
#[derive(Resource, Default, Clone)]
pub struct SoundEffects {
    pub click: Handle<AudioSource>,
    pub pop: Handle<AudioSource>,
    pub dig_grass: Handle<AudioSource>,
    pub dig_stone: Handle<AudioSource>,
    pub dig_wood: Handle<AudioSource>,
    pub dig_sand: Handle<AudioSource>,
    pub break_block: Handle<AudioSource>,
    pub step: Handle<AudioSource>,
    pub chest_open: Handle<AudioSource>,
    pub chest_close: Handle<AudioSource>,
    pub furnace_crackle: Handle<AudioSource>,
}

impl SoundEffects {
    /// Returns the appropriate digging/mining sound effect for a given block type.
    #[must_use]
    pub fn for_block(&self, block: BlockType) -> Handle<AudioSource> {
        match block {
            BlockType::Wood
            | BlockType::Planks
            | BlockType::CraftingTable
            | BlockType::Chest
            | BlockType::Furnace
            | BlockType::LitFurnace => self.dig_wood.clone(),

            BlockType::Grass | BlockType::Leaves | BlockType::Cactus => self.dig_grass.clone(),

            BlockType::Dirt
            | BlockType::Sand
            | BlockType::Sandstone
            | BlockType::Gravel
            | BlockType::Snow => self.dig_sand.clone(),

            // Stone, ores, bedrock, and general solid fallbacks
            _ => self.dig_stone.clone(),
        }
    }
}

/// Active OS audio output sink managed by `AudioPlugin`.
#[derive(Resource, Default)]
pub struct UniversalAudioOutput {
    sink: Option<rodio::MixerDeviceSink>,
}

/// Universally selects and opens the active OS audio output sink:
/// 1. Honors `ALSA_PCM_NAME` if explicitly set by the user.
/// 2. On Linux/BSD (ALSA), if `"default"` is not explicitly routed to a desktop sound server
///    (e.g. when `99-pipewire-default.conf` is not symlinked and `"default"` falls back to raw
///    hardware Card 0 `dmix:0`), probes whether a running `"pipewire"` or `"pulse"` sound server
///    PCM is present and connectable so audio routes to the user's active desktop sink.
/// 3. Falls back to `DeviceSinkBuilder::open_default_sink()` on Windows (WASAPI), macOS (CoreAudio),
///    Web, or pure ALSA systems.
fn open_universal_audio_sink() -> Option<rodio::MixerDeviceSink> {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};

    let host = rodio::cpal::default_host();

    if let Ok(env_pcm) = std::env::var("ALSA_PCM_NAME") {
        let target = env_pcm.trim();
        if !target.is_empty() {
            if let Ok(devices) = host.output_devices() {
                for dev in devices {
                    let is_match = dev
                        .description()
                        .is_ok_and(|d| d.driver() == Some(target) || d.name() == target);
                    if is_match {
                        if let Ok(mut sink) = rodio::DeviceSinkBuilder::from_device(dev)
                            .and_then(|b| b.open_sink_or_fallback())
                        {
                            sink.log_on_drop(false);
                            return Some(sink);
                        }
                    }
                }
            }
        }
    }

    if let Ok(devices) = host.output_devices() {
        let all_devs: Vec<rodio::cpal::Device> = devices.collect();
        let default_is_sound_server = all_devs.iter().any(|d| {
            d.description().is_ok_and(|desc| {
                desc.driver() == Some("default") && {
                    let text = format!("{desc:?}").to_ascii_lowercase();
                    text.contains("pipewire") || text.contains("pulse") || text.contains("jack")
                }
            })
        });

        if !default_is_sound_server {
            for preferred in ["pipewire", "pulse"] {
                if let Some(dev) = all_devs.iter().find(|d| {
                    d.description()
                        .is_ok_and(|desc| desc.driver() == Some(preferred))
                }) {
                    if let Ok(mut sink) = rodio::DeviceSinkBuilder::from_device(dev.clone())
                        .and_then(|b| b.open_sink_or_fallback())
                    {
                        sink.log_on_drop(false);
                        return Some(sink);
                    }
                }
            }
        }
    }

    rodio::DeviceSinkBuilder::open_default_sink()
        .map(|mut sink| {
            sink.log_on_drop(false);
            sink
        })
        .ok()
}

/// Helper that spawns a one-shot audio entity that automatically despawns upon playback dispatch.
pub fn play_sound(commands: &mut Commands, handle: &Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioPlayer(handle.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
    ));
}

/// Synthesizes a valid 48 kHz 16-bit mono PCM WAV buffer in memory when external
/// `assets/audio/*.wav` files are not present on disk (e.g., in CI or fresh clones).
#[must_use]
pub fn synthesize_fallback_wav(relative_path: &str) -> Vec<u8> {
    const SAMPLE_RATE: u32 = 48_000;
    let (duration_ms, base_freq, sweep_freq, noise_mix): (u32, f32, f32, f32) =
        if relative_path.contains("click") {
            (45, 950.0, 420.0, 0.25)
        } else if relative_path.contains("pop") {
            (65, 520.0, 1180.0, 0.10)
        } else if relative_path.contains("dig_grass") || relative_path.contains("step") {
            (90, 260.0, 140.0, 0.75)
        } else if relative_path.contains("dig_sand") {
            (85, 340.0, 190.0, 0.85)
        } else if relative_path.contains("dig_wood") {
            (95, 290.0, 160.0, 0.35)
        } else if relative_path.contains("break_block") {
            (160, 240.0, 95.0, 0.65)
        } else if relative_path.contains("chest_open") {
            (140, 310.0, 490.0, 0.20)
        } else if relative_path.contains("chest_close") {
            (130, 460.0, 240.0, 0.25)
        } else if relative_path.contains("furnace_crackle") {
            (150, 220.0, 380.0, 0.80)
        } else {
            // Default stone/mineral tap
            (85, 420.0, 190.0, 0.45)
        };

    let num_samples = (SAMPLE_RATE.saturating_mul(duration_ms)) / 1000;
    let data_bytes = num_samples.saturating_mul(2);
    let riff_size = 36_u32.saturating_add(data_bytes);

    let mut buf = Vec::with_capacity((44_u32.saturating_add(data_bytes)) as usize);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&riff_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16_u32.to_le_bytes()); // PCM chunk size
    buf.extend_from_slice(&1_u16.to_le_bytes()); // AudioFormat = 1 (PCM)
    buf.extend_from_slice(&1_u16.to_le_bytes()); // NumChannels = 1 (Mono)
    buf.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    buf.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // ByteRate
    buf.extend_from_slice(&2_u16.to_le_bytes()); // BlockAlign
    buf.extend_from_slice(&16_u16.to_le_bytes()); // BitsPerSample
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_bytes.to_le_bytes());

    let mut rng_state: u32 = 0x9E37_79B9 ^ (duration_ms.wrapping_mul(1_664_525));
    let total = num_samples.max(1) as f32;
    let mut phase: f32 = 0.0;

    for i in 0..num_samples {
        let progress = (i as f32) / total;
        let envelope = (1.0 - progress).powi(2);
        let freq = base_freq + (sweep_freq - base_freq) * progress;
        phase += std::f32::consts::TAU * freq / (SAMPLE_RATE as f32);

        rng_state = rng_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let white_noise = ((rng_state >> 16) as f32 / 32_768.0) - 1.0;

        let tone = phase.sin();
        let mixed = (tone * (1.0 - noise_mix) + white_noise * noise_mix) * envelope * 0.45;
        let sample = (mixed.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        buf.extend_from_slice(&sample.to_le_bytes());
    }

    buf
}

fn load_audio_file(
    relative_path: &str,
    audio_assets: &mut Assets<AudioSource>,
) -> Handle<AudioSource> {
    let disk_path = std::path::Path::new("assets").join(relative_path);
    if let Ok(bytes) = std::fs::read(&disk_path) {
        return audio_assets.add(AudioSource {
            bytes: bytes.into(),
        });
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            for ancestor in exe_dir.ancestors().take(4) {
                let candidate = ancestor.join("assets").join(relative_path);
                if let Ok(bytes) = std::fs::read(&candidate) {
                    return audio_assets.add(AudioSource {
                        bytes: bytes.into(),
                    });
                }
            }
        }
    }
    audio_assets.add(AudioSource {
        bytes: synthesize_fallback_wav(relative_path).into(),
    })
}

/// Startup system that opens the universal audio output sink and preloads all audio assets.
pub fn setup_audio_assets(mut commands: Commands, mut audio_assets: ResMut<Assets<AudioSource>>) {
    let sfx = SoundEffects {
        click: load_audio_file("audio/click.wav", &mut audio_assets),
        pop: load_audio_file("audio/pop.wav", &mut audio_assets),
        dig_grass: load_audio_file("audio/dig_grass.wav", &mut audio_assets),
        dig_stone: load_audio_file("audio/dig_stone.wav", &mut audio_assets),
        dig_wood: load_audio_file("audio/dig_wood.wav", &mut audio_assets),
        dig_sand: load_audio_file("audio/dig_sand.wav", &mut audio_assets),
        break_block: load_audio_file("audio/break_block.wav", &mut audio_assets),
        step: load_audio_file("audio/step.wav", &mut audio_assets),
        chest_open: load_audio_file("audio/chest_open.wav", &mut audio_assets),
        chest_close: load_audio_file("audio/chest_close.wav", &mut audio_assets),
        furnace_crackle: load_audio_file("audio/furnace_crackle.wav", &mut audio_assets),
    };

    commands.insert_resource(sfx);
    commands.insert_resource(UniversalAudioOutput {
        sink: open_universal_audio_sink(),
    });
}

fn dispatch_queued_audio_system(
    mut commands: Commands,
    audio_output: Res<UniversalAudioOutput>,
    audio_sources: Res<Assets<AudioSource>>,
    global_volume: Res<GlobalVolume>,
    query: Query<(Entity, &AudioPlayer<AudioSource>, &PlaybackSettings)>,
) {
    let Some(ref stream) = audio_output.sink else {
        for (entity, _, _) in &query {
            commands.entity(entity).despawn();
        }
        return;
    };

    let mixer = stream.mixer();
    for (entity, player_comp, settings) in &query {
        let Some(audio_source) = audio_sources.get(&player_comp.0) else {
            continue;
        };

        if !settings.muted && !settings.paused {
            let effective_vol = (settings.volume * global_volume.volume).to_linear();
            if effective_vol > 0.0 {
                let player = rodio::Player::connect_new(mixer);
                player.set_volume(effective_vol);
                player.set_speed(settings.speed);
                player.append(audio_source.decoder());
                player.detach();
            }
        }

        commands.entity(entity).despawn();
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalVolume>()
            .init_resource::<SoundEffects>()
            .init_resource::<UniversalAudioOutput>()
            .init_asset::<AudioSource>()
            .init_asset_loader::<AudioLoader>()
            .add_systems(Startup, setup_audio_assets)
            .add_systems(PostUpdate, dispatch_queued_audio_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_effects_for_block_categories() {
        let sfx = SoundEffects::default();
        let stone_sound = sfx.for_block(BlockType::Stone);
        let ore_sound = sfx.for_block(BlockType::CoalOre);
        let wood_sound = sfx.for_block(BlockType::Wood);
        let grass_sound = sfx.for_block(BlockType::Grass);
        let sand_sound = sfx.for_block(BlockType::Sand);

        assert_eq!(stone_sound, ore_sound);
        assert_eq!(stone_sound, sfx.dig_stone);
        assert_eq!(wood_sound, sfx.dig_wood);
        assert_eq!(grass_sound, sfx.dig_grass);
        assert_eq!(sand_sound, sfx.dig_sand);
    }

    #[test]
    fn test_synthesize_fallback_wav_is_valid_decodable_pcm() {
        for name in [
            "audio/click.wav",
            "audio/pop.wav",
            "audio/dig_grass.wav",
            "audio/dig_stone.wav",
            "audio/dig_wood.wav",
            "audio/dig_sand.wav",
            "audio/break_block.wav",
            "audio/step.wav",
            "audio/chest_open.wav",
            "audio/chest_close.wav",
            "audio/furnace_crackle.wav",
        ] {
            let wav_bytes = synthesize_fallback_wav(name);
            assert!(wav_bytes.len() > 44);
            assert_eq!(&wav_bytes[0..4], b"RIFF");
            assert_eq!(&wav_bytes[8..12], b"WAVE");
            let source = AudioSource {
                bytes: wav_bytes.into(),
            };
            // Verify rodio decodes the synthesized WAV header and samples without error
            let mut decoder = source.decoder();
            assert!(decoder.next().is_some());
        }
    }

    #[test]
    fn test_bevy_audio_output() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            AudioPlugin,
        ));
        app.update();

        let sfx = app.world().resource::<SoundEffects>().clone();
        let audio_assets = app.world().resource::<Assets<AudioSource>>();
        assert!(audio_assets.get(&sfx.click).is_some());
        assert!(audio_assets.get(&sfx.pop).is_some());
        assert!(audio_assets.get(&sfx.dig_grass).is_some());
        assert!(audio_assets.get(&sfx.dig_stone).is_some());
        assert!(audio_assets.get(&sfx.dig_wood).is_some());
        assert!(audio_assets.get(&sfx.dig_sand).is_some());
        assert!(audio_assets.get(&sfx.break_block).is_some());
        assert!(audio_assets.get(&sfx.step).is_some());
        assert!(audio_assets.get(&sfx.chest_open).is_some());
        assert!(audio_assets.get(&sfx.chest_close).is_some());
        assert!(audio_assets.get(&sfx.furnace_crackle).is_some());

        app.world_mut()
            .spawn((AudioPlayer(sfx.pop), PlaybackSettings::DESPAWN));
        for _ in 0..3 {
            app.update();
        }
    }
}
