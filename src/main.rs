use hound::{WavReader, WavWriter};
use std::f32::consts::PI;

const RING_MOD_PARAMS: RingModParams = RingModParams {
    mix: 71,
    frequency: 156.0,
    amount: 6.7,
    lfo_waveform: Waveform::Square,
    rate: 0.18,
};

enum Waveform {
    /// Sinusoidal LFO wave form will smoothly oscillate between 0-3 octaves above PARAMS.frequency
    Sinusoidal,
    /// Square LFO wave form instantaneously jumps between an unaffected carrier signal and 3 octaves above PARAMS.frequency
    Square,
}

struct RingModParams {
    /// LFO section
    /// 0 to 10, this is normalized and controls a percentage of a 3 octave jump
    amount: f32,
    /// the waveform for the carrier LFO modulation
    lfo_waveform: Waveform,
    /// 0.1Hz to 25Hz the rate of the LFO modulation on the carrier signal
    rate: f32,

    /// Modulator section
    /// 0 to 100, mix with the original sampled signal
    mix: u8,
    /// 0.6Hz to 80Hz (LO setting), 30Hz to 4kHz (HI setting) for the carrier signal
    frequency: f32,
}

fn ring_mod(
    sample_rate: u32,
    sample_length: usize,
    signal: impl IntoIterator<Item = i32>,
    params: &RingModParams,
) -> Vec<i32> {
    let mut res = vec![];

    // normalized mix and amount parameter
    let mix = f32::from(params.mix) / 100.0;
    let amount = params.amount / 10.0;

    // the signal
    let mut signal_iter = signal.into_iter();

    let mut lfo_phase = 0.0;
    let mut carrier_phase = 0.0;

    let lfo_increment = 2.0 * PI * params.rate / sample_rate as f32;

    for _ in 0..sample_length {
        lfo_phase = (lfo_phase + lfo_increment).rem_euclid(2.0 * PI);

        let lfo = match params.lfo_waveform {
            Waveform::Sinusoidal => lfo_phase.sin(),
            Waveform::Square => {
                if lfo_phase.sin() >= 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
        };

        // the carrier signal that's applied to the sampled one
        let carrier_increment =
            2.0 * PI * (params.frequency + lfo * (params.frequency * 3.0 * amount))
                / sample_rate as f32;

        carrier_phase = (carrier_phase + carrier_increment).rem_euclid(2.0 * PI);

        let carrier = carrier_phase.sin();

        if let Some(sample) = signal_iter.next() {
            let sample = sample as f32;

            // accounted for the mix parameter
            // see https://en.wikipedia.org/wiki/Ring_modulation#Simplified_operation
            let out_sample = (sample * (1.0 - mix)) + (sample * carrier * mix);

            res.push(out_sample as i32);
        } else {
            println!("Signal processing may be incomplete");
            break;
        }
    }

    res
}

fn main() {
    let r = WavReader::open("guitar.wav").unwrap();
    let mut w = WavWriter::create("output.wav", r.spec()).unwrap();

    // total number of samples in the input file "guitar.wav"
    let len = r.len();
    let sample_rate = r.spec().sample_rate;

    // the actual signal
    let signal = r
        .into_samples()
        .map(|sample| sample.expect("Failed to open signal as an array"))
        .collect::<Vec<i32>>();

    let ring_mod_result = ring_mod(sample_rate, len as usize, signal, &RING_MOD_PARAMS);

    for sample in ring_mod_result {
        w.write_sample(sample).unwrap();
    }
}
