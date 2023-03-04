/*
MIT License

Copyright (c) 2023 Philipp Schuster

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

#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from
)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]

use audio_visualizer::spectrum::plotters_png_file::spectrum_static_plotters_png_visualize;
use minimp3::{Decoder as Mp3Decoder, Error as Mp3Error, Frame as Mp3Frame};
use spectrum_analyzer::scaling::scale_to_zero_to_one;
use spectrum_analyzer::windows::{blackman_harris_4term, hamming_window, hann_window};
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use std::fs::File;
use std::time::Instant;

const TEST_OUT_DIR: &str = "test/out";

fn main() {
    println!("bass drum example:");
    example__bass_drum_sample();
    println!("============================");
    println!("clap beat example:");
    example__clap_beat_sample();
    println!("============================");
    println!("high hat example:");
    example__high_hat_sample();
}

#[allow(non_snake_case)]
fn example__bass_drum_sample() {
    // this sample is exactly 0,628s long
    // we have 44100samples/s*0,628s == 28695 samples
    // next smaller power of two is: 2^14 == 16384 => FFT needs power of 2
    let (samples, sampling_rate) =
        read_mp3_to_mono("test/samples/bass_drum_with_high-hat_at_end-sample.mp3");
    let samples = samples.into_iter().map(|x| x as f32).collect::<Vec<f32>>();

    to_spectrum_and_plot(
        &samples[0..4096],
        sampling_rate,
        "example__mp3-samples__bass_drum__spectrum",
        FrequencyLimit::Max(5000.0),
    )
}

#[allow(non_snake_case)]
fn example__clap_beat_sample() {
    // this sample is exactly 0,379s long
    // we have 44100samples/s*0,379s == 16714 samples
    // next smaller power of two is: 2^14 == 16384 => FFT needs power of 2
    let (samples, sampling_rate) = read_mp3_to_mono("test/samples/clap-beat-sample.mp3");
    let samples = samples.into_iter().map(|x| x as f32).collect::<Vec<f32>>();

    to_spectrum_and_plot(
        &samples[0..4096],
        sampling_rate,
        "example__mp3-samples__clap_beat__spectrum",
        FrequencyLimit::Max(5000.0),
    )
}

#[allow(non_snake_case)]
fn example__high_hat_sample() {
    // this sample is exactly 0,149s long
    // we have 44100samples/s*0,149s == 6571 samples
    // next smaller power of two is: 2^12 == 4096 => FFT needs power of 2
    let (samples, sampling_rate) = read_mp3_to_mono("test/samples/high-hat-sample.mp3");
    let samples = samples.into_iter().map(|x| x as f32).collect::<Vec<f32>>();

    to_spectrum_and_plot(
        &samples[0..4096],
        sampling_rate,
        "example__mp3-samples__high-hat__spectrum",
        FrequencyLimit::All,
    )
}

// Calculates spectrum via FFT for a given set of samples and applies
// all window functions + plots all
fn to_spectrum_and_plot(
    samples: &[f32],
    sampling_rate: u32,
    filename: &str,
    frequency_limit: FrequencyLimit,
) {
    let no_window = samples;

    let now = Instant::now();
    let hann_window = hann_window(no_window);
    println!(
        "[Measurement]: Hann-Window with {} samples took: {}µs",
        samples.len(),
        now.elapsed().as_micros()
    );
    let now = Instant::now();
    let hamming_window = hamming_window(no_window);
    println!(
        "[Measurement]: Hamming-Window with {} samples took: {}µs",
        samples.len(),
        now.elapsed().as_micros()
    );
    let blackman_harris_4term_window = blackman_harris_4term(no_window);
    println!(
        "[Measurement]: Blackmann-Harris-4-term-Window with {} samples took: {}µs",
        samples.len(),
        now.elapsed().as_micros()
    );
    let blackman_harris_7term_window = blackman_harris_4term(no_window);
    println!(
        "[Measurement]: Blackmann-Harris-7-term-Window with {} samples took: {}µs",
        samples.len(),
        now.elapsed().as_micros()
    );

    let now = Instant::now();
    let spectrum_no_window = samples_fft_to_spectrum(
        no_window,
        sampling_rate,
        frequency_limit,
        Some(&scale_to_zero_to_one),
    )
    .unwrap();
    println!(
        "[Measurement]: FFT to Spectrum with no window with {} samples took: {}µs",
        samples.len(),
        now.elapsed().as_micros()
    );
    let now = Instant::now();
    let spectrum_hamming_window = samples_fft_to_spectrum(
        &hamming_window,
        sampling_rate,
        frequency_limit,
        Some(&scale_to_zero_to_one),
    )
    .unwrap();
    println!(
        "[Measurement]: FFT to Spectrum with Hamming window with {} samples took: {}µs",
        samples.len(),
        now.elapsed().as_micros()
    );
    let now = Instant::now();
    let spectrum_hann_window = samples_fft_to_spectrum(
        &hann_window,
        sampling_rate,
        frequency_limit,
        Some(&scale_to_zero_to_one),
    )
    .unwrap();
    println!(
        "[Measurement]: FFT to Spectrum with Hann window with {} samples took: {}µs",
        samples.len(),
        now.elapsed().as_micros()
    );

    // println!("max is {:?}", spectrum_hann_window.max());

    let now = Instant::now();
    let spectrum_blackman_harris_4term_window = samples_fft_to_spectrum(
        &blackman_harris_4term_window,
        sampling_rate,
        frequency_limit,
        Some(&scale_to_zero_to_one),
    )
    .unwrap();
    println!("[Measurement]: FFT to Spectrum with Blackmann Harris 4-term window with {} samples took: {}µs", samples.len(), now.elapsed().as_micros());
    let now = Instant::now();
    let spectrum_blackman_harris_7term_window = samples_fft_to_spectrum(
        &blackman_harris_7term_window,
        sampling_rate,
        frequency_limit,
        Some(&scale_to_zero_to_one),
    )
    .unwrap();
    println!("[Measurement]: FFT to Spectrum with Blackmann Harris 7-term window with {} samples took: {}µs", samples.len(), now.elapsed().as_micros());

    /*for (fr, fr_val) in spectrum_hamming_window.data().iter() {
        println!("{}Hz => {}", fr, fr_val)
    }*/

    spectrum_static_plotters_png_visualize(
        &spectrum_no_window.to_map(),
        TEST_OUT_DIR,
        &format!("{}--no-window.png", filename),
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_hamming_window.to_map(),
        TEST_OUT_DIR,
        &format!("{}--hamming-window.png", filename),
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_hann_window.to_map(),
        TEST_OUT_DIR,
        &format!("{}--hann-window.png", filename),
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_blackman_harris_4term_window.to_map(),
        TEST_OUT_DIR,
        &format!("{}--blackman-harris-4-term-window.png", filename),
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_blackman_harris_7term_window.to_map(),
        TEST_OUT_DIR,
        &format!("{}--blackman-harris-7-term-window.png", filename),
    );
}

/// Reads an MP3 and returns the audio data as mono channel + the sample rate in Hertz.
fn read_mp3_to_mono(file: &str) -> (Vec<i16>, u32) {
    let mut decoder = Mp3Decoder::new(File::open(file).unwrap());

    let mut sampling_rate = 0;
    let mut mono_samples = vec![];
    loop {
        match decoder.next_frame() {
            Ok(Mp3Frame {
                data: samples_of_frame,
                sample_rate,
                channels,
                ..
            }) => {
                // that's a bird weird of the original API. Why should channels or sampling
                // rate change from frame to frame?

                // Should be constant throughout the MP3 file.
                sampling_rate = sample_rate;

                if channels == 2 {
                    for (i, sample) in samples_of_frame.iter().enumerate().step_by(2) {
                        let sample = *sample as i32;
                        let next_sample = samples_of_frame[i + 1] as i32;
                        mono_samples.push(((sample + next_sample) as f32 / 2.0) as i16);
                    }
                } else if channels == 1 {
                    mono_samples.extend_from_slice(&samples_of_frame);
                } else {
                    panic!("Unsupported number of channels={}", channels);
                }
            }
            Err(Mp3Error::Eof) => break,
            Err(e) => panic!("{:?}", e),
        }
    }

    (mono_samples, sampling_rate as u32)
}
