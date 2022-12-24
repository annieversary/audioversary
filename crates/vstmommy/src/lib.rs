use atomic_float::AtomicF32;
use ebur128::{EbuR128, Mode};
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod editor;

/// The time it takes for the peak meter to decay by 12 dB after switching to complete silence.
const PEAK_METER_DECAY_MS: f64 = 150.0;

/// This is mostly identical to the gain example, minus some fluff, and with a GUI.
pub struct Gain {
    params: Arc<GainParams>,

    /// Needed to normalize the peak meter's response based on the sample rate.
    peak_meter_decay_weight: f32,
    /// The current data for the peak meter. This is stored as an [`Arc`] so we can share it between
    /// the GUI and the audio processing parts. If you have more state to share, then it's a good
    /// idea to put all of that in a struct behind a single `Arc`.
    ///
    /// This is stored as voltage gain.
    peak_meter_l: Arc<AtomicF32>,
    peak_meter_r: Arc<AtomicF32>,

    lufs_meter: Arc<AtomicF32>,

    lufs_analyzer: EbuR128,
}

#[derive(Params)]
struct GainParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
}

impl Default for Gain {
    fn default() -> Self {
        Self {
            params: Arc::new(GainParams::default()),

            peak_meter_decay_weight: 1.0,

            peak_meter_l: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
            peak_meter_r: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
            lufs_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),

            // this doesn't matter, gets overwritten in initialize
            lufs_analyzer: EbuR128::new(2, 48000, Mode::M).unwrap(),
        }
    }
}

impl Default for GainParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
        }
    }
}

impl Plugin for Gain {
    const NAME: &'static str = "vst-mommy";
    const VENDOR: &'static str = "audioversary";
    const URL: &'static str = "https://audio.versary.town";
    const EMAIL: &'static str = "annie@versary.town";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const DEFAULT_INPUT_CHANNELS: u32 = 2;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 2;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
            self.peak_meter_l.clone(),
            self.peak_meter_r.clone(),
            self.lufs_meter.clone(),
        )
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        // This works with any symmetrical IO layout
        config.num_input_channels == config.num_output_channels && config.num_input_channels > 0
    }

    fn initialize(
        &mut self,
        _bus_config: &BusConfig,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // After `PEAK_METER_DECAY_MS` milliseconds of pure silence, the peak meter's value should
        // have dropped by 12 dB
        self.peak_meter_decay_weight = 0.25f64
            .powf((buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip())
            as f32;

        self.lufs_analyzer = EbuR128::new(2, buffer_config.sample_rate as u32, Mode::M).unwrap();

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for mut channel_samples in buffer.iter_samples() {
            let mut amplitude = [0.0; 2];
            let num_samples = channel_samples.len();

            for (id, sample) in channel_samples.iter_mut().enumerate() {
                amplitude[id] += *sample;
            }

            // TODO we're doing this for *every* sample pair, which is probably too much
            // do we move this outside the loop?
            // not sure how big num_samples will typically be

            if self.params.editor_state.is_open() {
                // left
                amplitude[0] = (amplitude[0] / num_samples as f32).abs();
                let current_peak_meter =
                    self.peak_meter_l.load(std::sync::atomic::Ordering::Relaxed);
                let new_peak_meter = if amplitude[0] > current_peak_meter {
                    amplitude[0]
                } else {
                    current_peak_meter * self.peak_meter_decay_weight
                        + amplitude[0] * (1.0 - self.peak_meter_decay_weight)
                };

                self.peak_meter_l
                    .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed);

                // right
                amplitude[1] = (amplitude[1] / num_samples as f32).abs();
                let current_peak_meter =
                    self.peak_meter_r.load(std::sync::atomic::Ordering::Relaxed);
                let new_peak_meter = if amplitude[1] > current_peak_meter {
                    amplitude[0]
                } else {
                    current_peak_meter * self.peak_meter_decay_weight
                        + amplitude[1] * (1.0 - self.peak_meter_decay_weight)
                };

                self.peak_meter_r
                    .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed);
            }
        }

        // lufs

        // Safety: ¯\_(ツ)_/¯
        self.lufs_analyzer
            .add_frames_planar_f32(unsafe { std::mem::transmute(buffer.as_slice_immutable()) })
            .unwrap();

        if let Ok(v) = self.lufs_analyzer.loudness_momentary() {
            self.lufs_meter
                .store(v as f32, std::sync::atomic::Ordering::Relaxed);
        }

        ProcessStatus::Normal
    }
}

// impl ClapPlugin for Gain {
//     const CLAP_ID: &'static str = "com.moist-plugins-gmbh.gain-gui-vizia";
//     const CLAP_DESCRIPTION: Option<&'static str> = Some("A smoothed gain parameter example plugin");
//     const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
//     const CLAP_SUPPORT_URL: Option<&'static str> = None;
//     const CLAP_FEATURES: &'static [ClapFeature] = &[
//         ClapFeature::AudioEffect,
//         ClapFeature::Stereo,
//         ClapFeature::Mono,
//         ClapFeature::Utility,
//     ];
// }

impl Vst3Plugin for Gain {
    const VST3_CLASS_ID: [u8; 16] = *b"VERSARYvst-mommy";
    const VST3_CATEGORIES: &'static str = "Fx|Analyzer";
}

// nih_export_clap!(Gain);
nih_export_vst3!(Gain);
