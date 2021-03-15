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
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, SpectrumTotalScaleFunctionFactory};
use spectrum_analyzer::windows::{blackman_harris_4term, hann_window, hamming_window};
use audio_visualizer::spectrum::staticc::plotters_png_file::spectrum_static_plotters_png_visualize;
use audio_visualizer::test_support::TEST_OUT_DIR;
use std::time::Instant;

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
    let samples = read_mp3("test/samples/bass_drum_with_high-hat_at_end-sample.mp3");

    to_spectrum_and_plot(
        &samples[0..16384],
        "example__mp3-samples__bass_drum__spectrum",
        FrequencyLimit::Max(5000.0)
    )
}

#[allow(non_snake_case)]
fn example__clap_beat_sample() {
    // this sample is exactly 0,379s long
    // we have 44100samples/s*0,379s == 16714 samples
    // next smaller power of two is: 2^14 == 16384 => FFT needs power of 2
    let samples = read_mp3("test/samples/clap-beat-sample.mp3");

    to_spectrum_and_plot(
        &samples[0..16384],
        "example__mp3-samples__clap_beat__spectrum",
        FrequencyLimit::Max(5000.0)
    )
}

#[allow(non_snake_case)]
fn example__high_hat_sample() {
    // this sample is exactly 0,149s long
    // we have 44100samples/s*0,149s == 6571 samples
    // next smaller power of two is: 2^12 == 4096 => FFT needs power of 2
    let samples = read_mp3("test/samples/high-hat-sample.mp3");

    to_spectrum_and_plot(
        &samples[0..4096],
        "example__mp3-samples__high-hat__spectrum",
        FrequencyLimit::All
    )
}

// Calculates spectrum via FFT for a given set of samples and applies
// all window functions + plots all
fn to_spectrum_and_plot(samples: &[f32], filename: &str, frequency_limit: FrequencyLimit) {
    let no_window = &samples[..];

    let now = Instant::now();
    let hann_window = hann_window(&no_window);
    println!("[Measurement]: Hann-Window with {} samples took: {}µs", samples.len(), now.elapsed().as_micros());
    let now = Instant::now();
    let hamming_window = hamming_window(&no_window);
    println!("[Measurement]: Hamming-Window with {} samples took: {}µs", samples.len(), now.elapsed().as_micros());
    let blackman_harris_4term_window = blackman_harris_4term(&no_window);
    println!("[Measurement]: Blackmann-Harris-4-term-Window with {} samples took: {}µs", samples.len(), now.elapsed().as_micros());
    let blackman_harris_7term_window = blackman_harris_4term(&no_window);
    println!("[Measurement]: Blackmann-Harris-7-term-Window with {} samples took: {}µs", samples.len(), now.elapsed().as_micros());

    let now = Instant::now();
    let spectrum_no_window = samples_fft_to_spectrum(
        &no_window,
        44100,
        frequency_limit,
        None,
        Some(get_scale_to_one_fn_factory()),
    );
    println!("[Measurement]: FFT to Spectrum with no window with {} samples took: {}µs", samples.len(), now.elapsed().as_micros());
    let now = Instant::now();
    let spectrum_hamming_window = samples_fft_to_spectrum(
        &hamming_window,
        44100,
        frequency_limit,
        None,
        Some(get_scale_to_one_fn_factory()),
    );
    println!("[Measurement]: FFT to Spectrum with Hamming window with {} samples took: {}µs", samples.len(), now.elapsed().as_micros());
    let now = Instant::now();
    let spectrum_hann_window = samples_fft_to_spectrum(
        &hann_window,
        44100,
        frequency_limit,
        None,
        Some(get_scale_to_one_fn_factory()),
    );
    println!("[Measurement]: FFT to Spectrum with Hann window with {} samples took: {}µs", samples.len(), now.elapsed().as_micros());
    let now = Instant::now();
    let spectrum_blackman_harris_4term_window = samples_fft_to_spectrum(
        &blackman_harris_4term_window,
        44100,
        frequency_limit,
        None,
        Some(get_scale_to_one_fn_factory()),
    );
    println!("[Measurement]: FFT to Spectrum with Blackmann Harris 4-term window with {} samples took: {}µs", samples.len(), now.elapsed().as_micros());
    let now = Instant::now();
    let spectrum_blackman_harris_7term_window = samples_fft_to_spectrum(
        &blackman_harris_7term_window,
        44100,
        frequency_limit,
        None,
        Some(get_scale_to_one_fn_factory()),
    );
    println!("[Measurement]: FFT to Spectrum with Blackmann Harris 7-term window with {} samples took: {}µs", samples.len(), now.elapsed().as_micros());

    /*for (fr, fr_val) in spectrum_hamming_window.data().iter() {
        println!("{}Hz => {}", fr, fr_val)
    }*/

    spectrum_static_plotters_png_visualize(
        &spectrum_no_window.to_map(None),
        TEST_OUT_DIR,
        &format!("{}--no-window.png", filename),
        false,
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_hamming_window.to_map(None),
        TEST_OUT_DIR,
        &format!("{}--hamming-window.png", filename),
        false,
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_hann_window.to_map(None),
        TEST_OUT_DIR,
        &format!("{}--hann-window.png", filename),
        false,
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_blackman_harris_4term_window.to_map(None),
        TEST_OUT_DIR,
        &format!("{}--blackman-harris-4-term-window.png", filename),
        false,
    );

    spectrum_static_plotters_png_visualize(
        &spectrum_blackman_harris_7term_window.to_map(None),
        TEST_OUT_DIR,
        &format!("{}--blackman-harris-7-term-window.png", filename),
        false,
    );
}

fn get_scale_to_one_fn_factory() -> SpectrumTotalScaleFunctionFactory{
    Box::new(
        move |_min: f32, max: f32, _average: f32, _median: f32| {
            Box::new(
                move |x| x/max
            )
        }
    )
}

fn read_mp3(file: &str) -> Vec<f32> {
    let samples = read_mp3_to_mono(file);
    let samples = samples.into_iter()
        .map(|x| x as f32)
        .collect::<Vec<f32>>();

    samples
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
