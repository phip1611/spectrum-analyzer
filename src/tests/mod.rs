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
use crate::tests::sine::sine_wave_audio_data_multiple;
use crate::windows::{blackman_harris_4term, blackman_harris_7term, hamming_window, hann_window};
use crate::{samples_fft_to_spectrum, FrequencyLimit, ComplexSpectrumScalingFunction};
use alloc::boxed::Box;
use alloc::vec::Vec;
use audio_visualizer::spectrum::staticc::plotters_png_file::spectrum_static_plotters_png_visualize;
use audio_visualizer::waveform::staticc::plotters_png_file::waveform_static_plotters_png_visualize;
use audio_visualizer::Channels;

/// Directory with test samples (e.g. mp3) can be found here.
#[allow(dead_code)]
const TEST_SAMPLES_DIR: &str = "test/samples";
/// If tests create files, they should be stored here.
#[allow(dead_code)]
const TEST_OUT_DIR: &str = "test/out";

mod sine;

#[test]
fn test_spectrum_and_visualize_sine_waves_50_1000_3777hz() {
    let sine_audio = sine_wave_audio_data_multiple(
        &[50.0, 1000.0, 3777.0], 44100, 1000
    );

    // visualize waveform
    waveform_static_plotters_png_visualize(
        &sine_audio,
        Channels::Mono,
        TEST_OUT_DIR,
        "test_spectrum_and_visualize_sine_waves_50_1000_3777hz--WAVEFORM.png",
    );

    let sine_audio = sine_audio
        .into_iter()
        .map(|x| x as f32)
        .collect::<Vec<f32>>();

    // FFT frequency accuracy is: sample_rate / (N / 2)
    // 44100/(4096/2) = 21.5Hz

    // get a window that we want to analyze
    // 1/44100 * 4096 => 0.0928s
    let window = &sine_audio[0..4096];

    let no_window = &window[..];
    let hamming_window = hamming_window(no_window);
    let hann_window = hann_window(no_window);

    let spectrum_no_window = samples_fft_to_spectrum(
        &no_window,
        44100,
        FrequencyLimit::Max(4000.0),
        None,
        Some(get_scale_to_one_fn_factory()),
    );

    let spectrum_hann_window = samples_fft_to_spectrum(
        &hann_window,
        44100,
        FrequencyLimit::Max(4000.0),
        None,
        Some(get_scale_to_one_fn_factory()),
    );

    let spectrum_hamming_window = samples_fft_to_spectrum(
        &hamming_window,
        44100,
        FrequencyLimit::Max(4000.0),
        None,
        Some(get_scale_to_one_fn_factory()),
    );

    spectrum_static_plotters_png_visualize(
        // spectrum_static_png_visualize(
        &spectrum_no_window.to_map(None),
        TEST_OUT_DIR,
        "test_spectrum_and_visualize_sine_waves_50_1000_3777hz--no-window.png",
        false,
    );

    spectrum_static_plotters_png_visualize(
        // spectrum_static_png_visualize(
        &spectrum_hamming_window.to_map(None),
        TEST_OUT_DIR,
        "test_spectrum_and_visualize_sine_waves_50_1000_3777hz--hamming-window.png",
        false,
    );

    spectrum_static_plotters_png_visualize(
        // spectrum_static_png_visualize(
        &spectrum_hann_window.to_map(None),
        TEST_OUT_DIR,
        "test_spectrum_and_visualize_sine_waves_50_1000_3777hz--hann-window.png",
        false,
    );

    // test getters match spectrum
    // we use Hann windowed spectrum because the accuracy is much better than
    // with no window!
    assert!(spectrum_hann_window.freq_val_exact(50.0).val() > 0.85);
    assert!(spectrum_hann_window.freq_val_closest(50.0).1.val() > 0.85);
    assert!(spectrum_hann_window.freq_val_exact(1000.0).val() > 0.85);
    assert!(spectrum_hann_window.freq_val_closest(1000.0).1.val() > 0.85);
    assert!(spectrum_hann_window.freq_val_exact(3777.0).val() > 0.85);
    assert!(spectrum_hann_window.freq_val_closest(3777.0).1.val() > 0.85);
    assert!(spectrum_hann_window.freq_val_exact(500.0).val() < 0.00001);
    assert!(spectrum_hann_window.freq_val_closest(500.0).1.val() < 0.00001);

    /*for (fr, vol) in spectrum.iter() {
        // you will experience inaccuracies here
        // TODO add further smoothing / noise reduction
        if *fr > 45.0.into() && *fr < 55.0.into() {
            println!("{}Hz => {}", fr, vol);
        }
    }*/
}

/// Tests that the spectrum contains the Nyquist frequency.
#[test]
fn test_spectrum_nyquist_theorem() {
    let dummy_audio_samples = vec![0.0; 4096];
    let spectrum = samples_fft_to_spectrum(
        &dummy_audio_samples,
        44100,
        FrequencyLimit::All,
        None,
        None,
    );
    assert_eq!(
        // because indices 0..N/2 (inclusive) are relevant
        4096 / 2 + 1,
        spectrum.data().iter()
            .map(|x| x.1)
            .filter(|x| x.val() == 0.0)
            .count(),
        "All frequency values must be exactly zero because the input signal is zero!"
    );
    assert_eq!(0.0, spectrum.min_fr().val(), "Maximum frequency must be Nyquist 0 Hz (DS Component/DC bias/Gleichwert)");
    assert_eq!(44100.0 / 2.0, spectrum.max_fr().val(), "Maximum frequency must be Nyquist frequency");
}
/// Tests that the spectrum contains the Nyquist frequency using a sine wave at almost Nyquist
/// frequency.
#[test]
fn test_spectrum_nyquist_theorem2() {
    let sine_audio = sine_wave_audio_data_multiple(
        // 22050.0 results in aliasing and no good results
        &[22049.9], 44100, 1000
    ).into_iter().map(|x| x as f32).collect::<Vec<f32>>();
    let spectrum = samples_fft_to_spectrum(
        &sine_audio[0..32768],
        44100,
        FrequencyLimit::All,
        None,
        Some(get_scale_to_one_fn_factory()),
    );
    assert_eq!(0.0, spectrum.min_fr().val(), "Maximum frequency must be Nyquist 0 Hz (DS Component/DC bias/Gleichwert)");
    assert_eq!(44100.0 / 2.0, spectrum.max_fr().val(), "Maximum frequency must be Nyquist frequency");
    assert!(spectrum.max().1.val() > 0.99, "Nyquist frequency must have a notable peak");
    // because I use 32768 samples, the frequency resolution is really good
    println!("{}", spectrum.freq_val_exact(22040.0));
    assert!(spectrum.freq_val_exact(22049.9).val() >= 0.94, "Other frequencies must not be part of the spectrum!");
    assert!(spectrum.freq_val_exact(22049.0).val() >= 0.49, "Other frequencies must not be part of the spectrum!");
    assert!(spectrum.freq_val_exact(22040.0).val() <= 0.05, "Other frequencies must not be part of the spectrum!");
    assert!(spectrum.freq_val_exact(22000.0).val() <= 0.01, "Other frequencies must not be part of the spectrum!");
}

fn get_scale_to_one_fn_factory() -> ComplexSpectrumScalingFunction {
    Box::new(move |_min: f32, max: f32, _average: f32, _median: f32| Box::new(move |x| x / max))
}
