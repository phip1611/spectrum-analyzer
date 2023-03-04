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
//! Test module for "integration"-like tests. No small unit tests of simple functions.

use crate::error::SpectrumAnalyzerError;
use crate::scaling::{divide_by_N, scale_to_zero_to_one};
use crate::tests::sine::sine_wave_audio_data_multiple;
use crate::windows::{hamming_window, hann_window};
use crate::{samples_fft_to_spectrum, FrequencyLimit};
use alloc::vec::Vec;
use audio_visualizer::spectrum::plotters_png_file::spectrum_static_plotters_png_visualize;
use audio_visualizer::waveform::plotters_png_file::waveform_static_plotters_png_visualize;
use audio_visualizer::Channels;
use core::cmp::max;

// /// Directory with test samples (e.g. mp3) can be found here.
// const TEST_SAMPLES_DIR: &str = "test/samples";
/// If tests create files, they should be stored here.
const TEST_OUT_DIR: &str = "test/out";

mod sine;

#[test]
fn test_spectrum_and_visualize_sine_waves_50_1000_3777hz() {
    let sine_audio = sine_wave_audio_data_multiple(&[50.0, 1000.0, 3777.0], 44100, 1000);

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

    let no_window = window;
    let hamming_window = hamming_window(no_window);
    let hann_window = hann_window(no_window);

    let spectrum_no_window = samples_fft_to_spectrum(
        no_window,
        44100,
        FrequencyLimit::Max(4000.0),
        Some(&scale_to_zero_to_one),
    )
    .unwrap();

    let spectrum_hann_window = samples_fft_to_spectrum(
        &hann_window,
        44100,
        FrequencyLimit::Max(4000.0),
        Some(&scale_to_zero_to_one),
    )
    .unwrap();

    let spectrum_hamming_window = samples_fft_to_spectrum(
        &hamming_window,
        44100,
        FrequencyLimit::Max(4000.0),
        Some(&scale_to_zero_to_one),
    )
    .unwrap();

    spectrum_static_plotters_png_visualize(
        // spectrum_static_png_visualize(
        &spectrum_no_window.to_map(),
        TEST_OUT_DIR,
        "test_spectrum_and_visualize_sine_waves_50_1000_3777hz--no-window.png",
    );

    spectrum_static_plotters_png_visualize(
        // spectrum_static_png_visualize(
        &spectrum_hamming_window.to_map(),
        TEST_OUT_DIR,
        "test_spectrum_and_visualize_sine_waves_50_1000_3777hz--hamming-window.png",
    );

    spectrum_static_plotters_png_visualize(
        // spectrum_static_png_visualize(
        &spectrum_hann_window.to_map(),
        TEST_OUT_DIR,
        "test_spectrum_and_visualize_sine_waves_50_1000_3777hz--hann-window.png",
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

/// This test is primarily for my personal understanding. It analyzes a specific constant
/// signal twice, but one time for twice the duration. If all FFT result values are
/// divided by their corresponding N (length of samples), the values must match
/// (with a small delta).
#[test]
fn test_spectrum_power() {
    let interesting_frequency = 2048.0;
    let sine_audio = sine_wave_audio_data_multiple(&[interesting_frequency], 44100, 1000);

    let sine_audio = sine_audio
        .into_iter()
        .map(|x| x as f32)
        .collect::<Vec<f32>>();

    // FFT frequency accuracy is: sample_rate / (N / 2)
    // 44100/(4096/2) = 21.5Hz

    // get a window that we want to analyze
    // 1/44100 * 4096 => 0.0928s
    let short_window = &sine_audio[0..2048];
    let long_window = &sine_audio[0..4096];
    //let very_long_window = &sine_audio[0..16384];

    let spectrum_short_window = samples_fft_to_spectrum(
        short_window,
        44100,
        FrequencyLimit::Max(4000.0),
        Some(&divide_by_N),
    )
    .unwrap();

    let spectrum_long_window = samples_fft_to_spectrum(
        long_window,
        44100,
        FrequencyLimit::Max(4000.0),
        Some(&divide_by_N),
    )
    .unwrap();

    /*let spectrum_very_long_window =
    samples_fft_to_spectrum(very_long_window, 44100, FrequencyLimit::Max(4000.0), Some(&divide_by_N_sqrt))
        .unwrap();*/

    spectrum_static_plotters_png_visualize(
        &spectrum_short_window.to_map(),
        TEST_OUT_DIR,
        "test_spectrum_power__short_window.png",
    );
    spectrum_static_plotters_png_visualize(
        &spectrum_long_window.to_map(),
        TEST_OUT_DIR,
        "test_spectrum_power__long_window.png",
    );
    /*spectrum_static_plotters_png_visualize(
        &spectrum_long_window.to_map(),
        TEST_OUT_DIR,
        "test_spectrum_power__very_long_window.png",
    );*/

    let a = spectrum_short_window.freq_val_exact(interesting_frequency);
    let b = spectrum_long_window.freq_val_exact(interesting_frequency);
    //let c = spectrum_very_long_window.freq_val_exact(interesting_frequency);
    //dbg!(a, b, c);
    let ab_abs_diff = (a - b).val().abs();
    //let ac_abs_diff = (a - c).val().abs();
    let ab_deviation = ab_abs_diff / max(a, b).val();
    //let ac_deviation = ac_abs_diff / max(a, c).val();
    assert!(
        ab_deviation < 0.122,
        "Values must more or less equal, because both were divided by their N. deviation={}",
        ab_deviation
    );
    //assert!(
    //    ac_deviation < 0.07,
    //    "Values must more or less equal, because both were divided by their N. deviation={}", ac_deviation
    //);
}

#[test]
fn test_spectrum_frequency_limit_inclusive() {
    let sampling_rate = 1024;
    let sine_audio = sine_wave_audio_data_multiple(&[512.0], sampling_rate, 1000);

    let sine_audio = sine_audio
        .into_iter()
        .map(|x| x as f32)
        .collect::<Vec<f32>>();

    // frequency resolution will be:
    // 1024 / 512 = 2 Hz
    // we use even frequency resolution in this example for easy testing
    // max detectable frequency here is 512Hz

    let window = hann_window(&sine_audio[0..512]);

    {
        let spectrum =
            samples_fft_to_spectrum(&window, sampling_rate, FrequencyLimit::Max(400.0), None)
                .unwrap();
        assert_eq!(
            spectrum.min_fr().val(),
            0.0,
            "Lower bound frequency must be inclusive!"
        );
        assert_eq!(
            spectrum.max_fr().val(),
            400.0,
            "Upper bound frequency must be inclusive!"
        );
    }
    {
        let spectrum =
            samples_fft_to_spectrum(&window, sampling_rate, FrequencyLimit::Min(100.0), None)
                .unwrap();
        assert_eq!(
            spectrum.min_fr().val(),
            100.0,
            "Lower bound frequency must be inclusive!"
        );
        assert_eq!(
            spectrum.max_fr().val(),
            sampling_rate as f32 / 2.0,
            "Upper bound frequency must be inclusive!"
        );
    }
    {
        let spectrum = samples_fft_to_spectrum(
            &window,
            sampling_rate,
            FrequencyLimit::Range(412.0, 510.0),
            None,
        )
        .unwrap();
        assert_eq!(
            spectrum.min_fr().val(),
            412.0,
            "Lower bound frequency must be inclusive!"
        );
        assert_eq!(
            spectrum.max_fr().val(),
            510.0,
            "Upper bound frequency must be inclusive!"
        );
    }
}

/// Tests that the spectrum contains the Nyquist frequency.
#[test]
fn test_spectrum_nyquist_theorem() {
    let dummy_audio_samples = vec![0.0; 4096];
    let spectrum =
        samples_fft_to_spectrum(&dummy_audio_samples, 44100, FrequencyLimit::All, None).unwrap();
    assert_eq!(
        // because indices 0..N/2 (inclusive) of the FFT result are relevant
        // => DC component to Nyquist frequency
        4096 / 2 + 1,
        spectrum
            .data()
            .iter()
            .map(|x| x.1)
            .filter(|x| x.val() == 0.0)
            .count(),
        "All frequency values must be exactly zero because the input signal is zero!"
    );
    assert_eq!(
        0.0,
        spectrum.min_fr().val(),
        "Minimum frequency must be 0 Hz (DS Component/DC bias/Gleichwert)"
    );
    assert_eq!(
        44100.0 / 2.0,
        spectrum.max_fr().val(),
        "Maximum frequency must be Nyquist frequency"
    );
}
/// Tests that the spectrum contains the Nyquist frequency using a sine wave at almost Nyquist
/// frequency.
#[test]
fn test_spectrum_nyquist_theorem2() {
    let sine_audio = sine_wave_audio_data_multiple(
        // 22050.0 results in aliasing and no good results
        &[22049.9],
        44100,
        1000,
    )
    .into_iter()
    .map(|x| x as f32)
    .collect::<Vec<f32>>();
    let spectrum = samples_fft_to_spectrum(
        &sine_audio[0..4096],
        44100,
        FrequencyLimit::All,
        Some(&scale_to_zero_to_one),
    )
    .unwrap();
    assert_eq!(
        0.0,
        spectrum.min_fr().val(),
        "Maximum frequency must be Nyquist 0 Hz (DS Component/DC bias/Gleichwert)"
    );
    assert_eq!(
        44100.0 / 2.0,
        spectrum.max_fr().val(),
        "Maximum frequency must be Nyquist frequency"
    );
    assert!(
        spectrum.max().1.val() > 0.99,
        "Nyquist frequency must have a notable peak"
    );

    // frequency resolution is: 44100/4096 = ~ 11hz
    assert!(
        spectrum.freq_val_exact(22049.9).val() >= 0.94,
        "Other frequencies must not be part of the spectrum!"
    );
    assert!(
        spectrum.freq_val_exact(22049.0).val() >= 0.49,
        "Other frequencies must not be part of the spectrum!"
    );
    assert!(
        spectrum.freq_val_exact(22035.0).val() <= 0.26,
        "Other frequencies must not be part of the spectrum!"
    );
    assert!(
        spectrum.freq_val_exact(22000.0).val() <= 0.07,
        "Other frequencies must not be part of the spectrum!"
    );
    assert!(
        spectrum.freq_val_exact(21500.0).val() <= 0.01,
        "Other frequencies must not be part of the spectrum!"
    );
}

#[test]
fn test_invalid_input() {
    // should not contain NaN
    let samples = vec![0.0, 1.0, 2.0, 3.0, f32::NAN, 4.0, 5.0, 6.0];
    let err = samples_fft_to_spectrum(&samples, 44100, FrequencyLimit::All, None).unwrap_err();
    assert!(matches!(err, SpectrumAnalyzerError::NaNValuesNotSupported));

    // should not contain Infinity
    let samples = vec![0.0, 1.0, 2.0, 3.0, f32::INFINITY, 4.0, 5.0, 6.0];
    let err = samples_fft_to_spectrum(&samples, 44100, FrequencyLimit::All, None).unwrap_err();
    assert!(matches!(
        err,
        SpectrumAnalyzerError::InfinityValuesNotSupported
    ));

    // needs at least two samples
    let samples = vec![0.0];
    let err = samples_fft_to_spectrum(&samples, 44100, FrequencyLimit::All, None).unwrap_err();
    assert!(matches!(err, SpectrumAnalyzerError::TooFewSamples));

    // test frequency limit gets verified
    let samples = vec![0.0; 4];
    let err =
        samples_fft_to_spectrum(&samples, 44100, FrequencyLimit::Min(-1.0), None).unwrap_err();
    assert!(matches!(
        err,
        SpectrumAnalyzerError::InvalidFrequencyLimit(_)
    ));

    // samples length not a power of two
    let samples = vec![0.0; 3];
    let err = samples_fft_to_spectrum(&samples, 44100, FrequencyLimit::All, None).unwrap_err();
    assert!(matches!(
        err,
        SpectrumAnalyzerError::SamplesLengthNotAPowerOfTwo
    ));
}

#[test]
fn test_only_null_samples_valid() {
    let samples = vec![0.0, 0.0];
    let _ = samples_fft_to_spectrum(&samples, 44100, FrequencyLimit::All, None).unwrap();
}

#[test]
fn test_scaling_produces_error() {
    let samples = vec![1.1, 2.2, 3.3, 4.4, 5.5, 6.6, 7.7, 8.8];
    let _ = samples_fft_to_spectrum(
        &samples,
        44100,
        FrequencyLimit::All,
        Some(&|_val, _info| f32::NAN),
    )
    .expect_err("Must throw error due to illegal scaling!");
}

/// Test that the scaling actually has the effect that we expect it to have.
#[test]
fn test_divide_by_n_has_effect() {
    let audio_data = sine_wave_audio_data_multiple(&[100.0, 200.0, 400.0], 1000, 2000);
    let audio_data = audio_data.into_iter().map(|x| x as f32).collect::<Vec<_>>();
    let audio_data = hann_window(&audio_data[0..1024]);
    let normal_spectrum =
        samples_fft_to_spectrum(&audio_data, 1000, FrequencyLimit::All, None).unwrap();
    let scaled_spectrum =
        samples_fft_to_spectrum(&audio_data, 1000, FrequencyLimit::All, Some(&divide_by_N))
            .unwrap();
    for i in 0..512 {
        let actual_no_scaling = normal_spectrum.data()[i].1.val();
        let actual_with_scaling = scaled_spectrum.data()[i].1.val();
        assert!(
            (actual_no_scaling / 1024.0 - actual_with_scaling) < 0.1,
            "[{}] actual_no_scaling={} should be roughly 1024 times bigger than actual_with_scaling={}",
            i,
            actual_no_scaling,
            actual_with_scaling,
        );
    }

    // now check that the "divide by N" also works, if there is a frequency limit
    let scaled_spectrum_with_limit = samples_fft_to_spectrum(
        &audio_data,
        1000,
        FrequencyLimit::Max(250.0),
        Some(&divide_by_N),
    )
    .unwrap();

    for i in 0..256 {
        let reference = scaled_spectrum.data()[i].1.val();
        let actual = scaled_spectrum_with_limit.data()[i].1.val();
        assert_eq!(
            reference, actual,
            "having less frequencies in the spectrum due to a limit must not effect N!"
        );
    }
}
