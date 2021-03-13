/*
MIT License

Copyright (c) 2021 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
#[macro_use]
extern crate std;
use std::fs::File;
use minimp3::{Decoder as Mp3Decoder, Frame as Mp3Frame, Error as Mp3Error};
use spectrum_analyzer::{hann_window, samples_fft_to_spectrum};

fn main() {
    println!("bass drum example:");
    bass_drum_sample();
    println!("============================");
    println!("clap beat example:");
    clap_beat_sample();
    println!("============================");
    println!("high hat example:");
    high_hat_sample();
}

fn bass_drum_sample() {
    // this sample is exactly 0,628s long
    // we have 44100samples/s*0,628s == 28695 samples
    // next smaller power of two is: 2^14 == 16384 => FFT needs power of 2
    let samples = read_mp3_to_mono("test-res/bass_drum_with_high-hat_at_end-sample.mp3");
    let samples = samples.into_iter()
        .map(|x| x as f32)
        .collect::<Vec<f32>>();
    let hann_window = hann_window(&samples[0..16384]);

    let spectrum = samples_fft_to_spectrum(
        &hann_window,
        44100,
        Some(&|x| 20.0 * x.log10()),
        None,
    );

    // we expect only the low frequencies to be visible here (<=100Hz)
    for (fr, vol) in spectrum.iter() {
        // TBH: value is not chosen systematically.
        // I just looked what dB value the highest values have.
        // TODO needs further explanation/more knowledge about the value
        if *vol > 130.0 {
            println!("{}Hz => {}", fr, vol);
        }
    }
}

fn clap_beat_sample() {
    // this sample is exactly 0,379s long
    // we have 44100samples/s*0,379s == 16714 samples
    // next smaller power of two is: 2^14 == 16384 => FFT needs power of 2
    let samples = read_mp3_to_mono("test-res/clap-beat-sample.mp3");
    let samples = samples.into_iter()
        .map(|x| x as f32)
        .collect::<Vec<f32>>();
    let hann_window = hann_window(&samples[0..16384]);

    let spectrum = samples_fft_to_spectrum(
        &hann_window,
        44100,
        Some(&|x| 20.0 * x.log10()),
        None,
    );

    // we expect not only low frequencies here
    for (fr, vol) in spectrum.iter() {
        // TBH: value is not chosen systematically.
        // I just looked what dB value the highest values have.
        // TODO needs further explanation/more knowledge about the value
        if *vol > 125.0 {
            println!("{}Hz => {}", fr, vol);
        }
    }
}

fn high_hat_sample() {
    // this sample is exactly 0,149s long
    // we have 44100samples/s*0,149s == 6571 samples
    // next smaller power of two is: 2^12 == 4096 => FFT needs power of 2

    let samples = read_mp3_to_mono("test-res/high-hat-sample.mp3");
    let samples = samples.into_iter()
        .map(|x| x as f32)
        .collect::<Vec<f32>>();
    let hann_window = hann_window(&samples[0..4096]);

    let spectrum = samples_fft_to_spectrum(
        &hann_window,
        44100,
        Some(&|x| 20.0 * x.log10()),
        None,
    );

    // we expect only the low frequencies to be visible here (<=100Hz)
    for (fr, vol) in spectrum.iter() {
        // TBH: value is not chosen systematically.
        // TODO needs further explanation/more knowledge about the value
        if *vol > 35.0 {
            println!("{}Hz => {}", fr, vol);
        }
    }

}


fn read_mp3_to_mono(file: &str) -> Vec<i16> {
    let mut decoder = Mp3Decoder::new(File::open(file).unwrap());

    let mut lrlr_mp3_samples = vec![];
    loop {
        match decoder.next_frame() {
            Ok(Mp3Frame { data: samples_of_frame, .. }) => {
                for sample in samples_of_frame {
                    lrlr_mp3_samples.push(sample);
                }
            }
            Err(Mp3Error::Eof) => break,
            Err(e) => panic!("{:?}", e),
        }
    }

    lrlr_mp3_samples
}
