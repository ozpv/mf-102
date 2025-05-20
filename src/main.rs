use hound::{WavReader, WavWriter};
use std::f32::consts::PI;

#[allow(unused)]
enum Waveform {
    /// Sinusoidal LFO wave form will smoothly oscillate between 0-3 octaves above PARAMS.frequency
    Sinusoidal,
    /// Square LFO wave form instantaneously jumps between an unaffected carrier signal and 3 octaves above PARAMS.frequency
    Square,
}

struct Params {
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

const PARAMS: Params = Params {
    mix: 71,
    frequency: 156.0,
    amount: 6.7,
    lfo_waveform: Waveform::Square,
    rate: 0.18,
};

fn main() {
    let r = WavReader::open("guitar.wav").unwrap();
    let mut w = WavWriter::create("output.wav", r.spec()).unwrap();

    // total number of samples in the input file "guitar.wav"
    let len = r.len();
    let sample_rate = r.spec().sample_rate;
    let mut samples_iter = r.into_samples::<i32>();

    // counts the current time into reader in seconds
    let mut time = 0.0;

    // normalized mix parameter
    let mix = PARAMS.mix as f32 / 100.0;

    for _ in 0..len {
        time += 1.0 / sample_rate as f32;

        let lfo = match PARAMS.lfo_waveform {
            Waveform::Sinusoidal => (2.0 * PI * PARAMS.rate * time).sin(),
            Waveform::Square => {
                if (2.0 * PI * PARAMS.rate * time).sin() >= 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
        };

        // the carrier signal that's applied to the sampled one
        let carrier = (2.0
            * PI
            * (PARAMS.frequency + lfo * (PARAMS.frequency * 3.0 * (PARAMS.amount / 10.0)))
            * time)
            .sin();

        if let Some(sample) = samples_iter.next().and_then(Result::ok) {
            let sample = sample as f32;

            // accounted for the mix parameter
            // see https://en.wikipedia.org/wiki/Ring_modulation#Simplified_operation
            let out_sample = (sample * (1.0 - mix)) + (sample * carrier * mix);

            w.write_sample(out_sample as i32).unwrap();
        } else {
            println!("Failed to write all samples");
            break;
        }
    }

    // dropping the writer writes to output.wav
    drop(w);

    println!("Wrote {len} samples to output.wav");
}
