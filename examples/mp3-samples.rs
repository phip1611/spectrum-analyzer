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
use spectrum_analyzer::scaling::scale_to_zero_to_one;
use spectrum_analyzer::windows::{blackman_harris_4term, hamming_window, hann_window};
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;
use symphonia::core::audio::{AudioBuffer, Signal};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

/// Returns the location where tests should store files they produce.
fn test_out_dir() -> PathBuf {
    let path = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let dir = PathBuf::from(dir);
            dir.join("target")
        });
    let path = path.join("test_generated");
    if !path.exists() {
        // This can fail, as tests are run in parallel.
        let _ = std::fs::create_dir(path.clone());
    }
    path
}

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
        test_out_dir().to_str().unwrap(),
        &format!("{filename}--no-window.png"),
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_hamming_window.to_map(),
        test_out_dir().to_str().unwrap(),
        &format!("{filename}--hamming-window.png"),
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_hann_window.to_map(),
        test_out_dir().to_str().unwrap(),
        &format!("{filename}--hann-window.png"),
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_blackman_harris_4term_window.to_map(),
        test_out_dir().to_str().unwrap(),
        &format!("{filename}--blackman-harris-4-term-window.png"),
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_blackman_harris_7term_window.to_map(),
        test_out_dir().to_str().unwrap(),
        &format!("{filename}--blackman-harris-7-term-window.png"),
    );
}

/// Reads an mp3 file and returns mono samples.
fn read_mp3_to_mono<P: AsRef<Path>>(file: P) -> (Vec<i16>, u32) {
    let file = File::open(file).unwrap();
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let probed = get_probe()
        .format(
            &Hint::default(),
            mss,
            &Default::default(),
            &Default::default(),
        )
        .unwrap();
    let mut format_reader = probed.format;
    let track = format_reader.tracks().first().unwrap();
    let mut decoder = get_codecs()
        .make(&track.codec_params, &Default::default())
        .unwrap();

    let mut audio_data_lrlr = Vec::new();
    let mut sampling_rate = None;
    while let Ok(packet) = format_reader.next_packet() {
        if let Ok(audio_buf_ref) = decoder.decode(&packet) {
            let audio_spec = audio_buf_ref.spec();
            if sampling_rate.is_none() {
                sampling_rate.replace(audio_spec.rate);
            }

            let mut audio_buf_i16 =
                AudioBuffer::<i16>::new(audio_buf_ref.frames() as u64, *audio_spec);
            audio_buf_ref.convert(&mut audio_buf_i16);

            match audio_spec.channels.count() {
                2 => {
                    let iter = audio_buf_i16
                        .chan(0)
                        .iter()
                        .zip(audio_buf_i16.chan(1))
                        // LRLR interleavment to mono
                        .map(|(&l, &r)| ((l as i32 + r as i32) / 2) as i16);
                    audio_data_lrlr.extend(iter);
                }
                n => panic!("Unsupported amount of channels: {n}"),
            }
        }
    }
    (audio_data_lrlr, sampling_rate.unwrap())
}
