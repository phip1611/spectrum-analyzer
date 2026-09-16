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
use crate::windows::{blackman_harris_4term, blackman_harris_7term, hamming_window, hann_window};
use crate::{FrequencyLimit, samples_fft_to_spectrum};
use alloc::vec::Vec;
use audio_visualizer::SpectrumVisualizer;
use audio_visualizer::WaveformVisualizer;
use core::cmp::max;
use core::f32::consts::PI;
use std::path::PathBuf;

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

mod sine;

#[test]
#[cfg_attr(miri, ignore)] // runs forever + no real value add
fn test_spectrum_and_visualize_sine_waves_50_1000_3777hz() {
    let sine_audio = sine_wave_audio_data_multiple(&[50.0, 1000.0, 3777.0], 44100, 1000);

    WaveformVisualizer::new(&sine_audio)
        .title("Waveform of Sine Waves (50 Hz, 1000 Hz, 3777 Hz)")
        .sample_rate(44100.0)
        .write_png(format!(
            "{}/test_spectrum_and_visualize_sine_waves_50_1000_3777hz--waveform.png",
            test_out_dir().display()
        ))
        .unwrap();

    // FFT frequency resolution is: sample_rate / N
    // 44100/4096 = 10.8Hz

    // get a window that we want to analyze
    // 1/44100 * 4096 => 0.0928s
    let window = &sine_audio[0..4096];

    let spectra = [
        ("no-window", window.to_vec()),
        ("hamming-window", hamming_window(window)),
        ("hann-window", hann_window(window)),
        (
            "blackman-harris-4-term-window",
            blackman_harris_4term(window),
        ),
        (
            "blackman-harris-7-term-window",
            blackman_harris_7term(window),
        ),
    ]
    .into_iter()
    .map(|(filename_suffix, samples)| {
        let frequency_spectrum = samples_fft_to_spectrum(
            &samples,
            44100,
            FrequencyLimit::Max(4000.0),
            Some(&scale_to_zero_to_one),
        )
        .unwrap();

        let spectrum_data = frequency_spectrum
            .data()
            .iter()
            .map(|(fr, fr_val)| (fr.val(), fr_val.val()))
            .collect::<Vec<_>>();

        // visualize waveform as png.
        SpectrumVisualizer::new(&spectrum_data)
            .title("Spectrum of Sine Waves (50 Hz, 1000 Hz, 3777 Hz)")
            .size(700, 700)
            .highlight(50.0)
            .highlight(1000.0)
            .highlight(3777.0)
            .write_png(format!(
                "{}/test_spectrum_and_visualize_sine_waves_50_1000_3777hz--{filename_suffix}.png",
                test_out_dir().display()
            ))
            .unwrap();

        (filename_suffix, frequency_spectrum)
    })
    .collect::<Vec<_>>();

    let spectrum_hann_window = &spectra
        .iter()
        .find(|(filename_suffix, _spectrum)| *filename_suffix == "hann-window")
        .unwrap()
        .1;

    // test getters match spectrum
    // we use Hann windowed spectrum because the accuracy is much better than
    // with no window!
    let exact = |fr| spectrum_hann_window.freq_val_exact(fr).unwrap().val();
    let closest = |fr| spectrum_hann_window.freq_val_closest(fr).unwrap().1.val();

    assert!(exact(50.0) > 0.85);
    assert!(closest(50.0) > 0.85);
    assert!(exact(1000.0) > 0.85);
    assert!(closest(1000.0) > 0.85);
    assert!(exact(3777.0) > 0.85);
    assert!(closest(3777.0) > 0.85);
    assert!(exact(500.0) < 0.0001);
    assert!(closest(500.0) < 0.0001);
}

/// This test is primarily for my personal understanding. It analyzes a specific constant
/// signal twice, but one time for twice the duration. If all FFT result values are
/// divided by their corresponding N (length of samples), the values must match
/// (with a small delta).
#[test]
#[cfg_attr(miri, ignore)] // runs forever + no real value add
fn test_spectrum_power() {
    let interesting_frequency = 2048.0;
    let sine_audio = sine_wave_audio_data_multiple(&[interesting_frequency], 44100, 1000);

    // FFT frequency resolution is: sample_rate / N
    // 44100/4096 = 10.8Hz

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

    // visualize waveform as png.
    let visualize_fn = |spectrum_data: &[(f32, f32)], filename_suffix: &str| {
        // visualize waveform as png.
        SpectrumVisualizer::new(spectrum_data)
            .title("Spectrum of Sine Wave (2048 Hz)")
            .size(700, 700)
            .highlight(2048.0)
            .write_png(format!(
                "{}/test_spectrum_power__{filename_suffix}.png",
                test_out_dir().display()
            ))
            .unwrap();
    };

    visualize_fn(&spectrum_short_window.to_vec(), "short_window");
    visualize_fn(&spectrum_long_window.to_vec(), "long_window");

    let a = spectrum_short_window
        .freq_val_exact(interesting_frequency)
        .unwrap();
    let b = spectrum_long_window
        .freq_val_exact(interesting_frequency)
        .unwrap();

    let ab_abs_diff = (a - b).val().abs();
    let ab_deviation = ab_abs_diff / max(a, b).val();
    assert!(
        ab_deviation < 0.122,
        "Values must more or less equal, because both were divided by their N. deviation={ab_deviation}"
    );
}

#[test]
fn test_spectrum_frequency_limit_inclusive() {
    let sampling_rate = 1024;
    let sine_audio = sine_wave_audio_data_multiple(&[512.0], sampling_rate, 1000);

    let sine_audio = sine_audio.into_iter().collect::<Vec<f32>>();

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
#[cfg_attr(miri, ignore)] // runs forever + no real value add
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
#[cfg_attr(miri, ignore)] // runs forever + no real value add
fn test_spectrum_nyquist_theorem2() {
    let sine_audio = sine_wave_audio_data_multiple(
        // 22050.0 results in aliasing and no good results
        &[22049.9],
        44100,
        1000,
    )
    .into_iter()
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
        spectrum.freq_val_exact(22049.9).unwrap().val() >= 0.94,
        "Other frequencies must not be part of the spectrum!"
    );
    assert!(
        spectrum.freq_val_exact(22049.0).unwrap().val() >= 0.49,
        "Other frequencies must not be part of the spectrum!"
    );
    assert!(
        spectrum.freq_val_exact(22035.0).unwrap().val() <= 0.26,
        "Other frequencies must not be part of the spectrum!"
    );
    assert!(
        spectrum.freq_val_exact(22000.0).unwrap().val() <= 0.07,
        "Other frequencies must not be part of the spectrum!"
    );
    assert!(
        spectrum.freq_val_exact(21500.0).unwrap().val() <= 0.01,
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
    assert!(matches!(err, SpectrumAnalyzerError::InvalidLengthOfSamples));

    // max 32768 (microfft limitation)
    let samples = vec![0.0; 32768];
    let res = samples_fft_to_spectrum(&samples, 44100, FrequencyLimit::All, None);
    assert!(res.is_ok());

    // no more than 32768
    let samples = vec![0.0; 32769];
    let err = samples_fft_to_spectrum(&samples, 44100, FrequencyLimit::All, None).unwrap_err();
    assert!(matches!(err, SpectrumAnalyzerError::InvalidLengthOfSamples));

    // test frequency limit gets verified
    let samples = vec![0.0; 4];
    let err =
        samples_fft_to_spectrum(&samples, 44100, FrequencyLimit::Min(-1.0), None).unwrap_err();
    assert!(matches!(
        err,
        SpectrumAnalyzerError::InvalidFrequencyLimit(_)
    ));

    // frequency limits must leave at least two bins after filtering
    let samples = vec![0.0; 8];
    let err =
        samples_fft_to_spectrum(&samples, 8, FrequencyLimit::Range(1.1, 1.9), None).unwrap_err();
    assert!(matches!(
        err,
        SpectrumAnalyzerError::FrequencyLimitTooNarrow
    ));
    let err =
        samples_fft_to_spectrum(&samples, 8, FrequencyLimit::Range(1.0, 1.0), None).unwrap_err();
    assert!(matches!(
        err,
        SpectrumAnalyzerError::FrequencyLimitTooNarrow
    ));

    // samples length not a power of two
    let samples = vec![0.0; 3];
    let err = samples_fft_to_spectrum(&samples, 44100, FrequencyLimit::All, None).unwrap_err();
    assert!(matches!(err, SpectrumAnalyzerError::InvalidLengthOfSamples));
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
#[cfg_attr(miri, ignore)] // runs forever + no real value add
fn test_divide_by_n_has_effect() {
    let audio_data = sine_wave_audio_data_multiple(&[100.0, 200.0, 400.0], 1000, 2000);
    let audio_data = audio_data.into_iter().collect::<Vec<_>>();
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
            "[{i}] actual_no_scaling={actual_no_scaling} should be roughly 1024 times bigger than actual_with_scaling={actual_with_scaling}",
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

/// Checks the relation between input and spectrum values documented on
/// [`samples_fft_to_spectrum`]: a sine wave with amplitude `A` on a bin
/// frequency yields `A * N / 2`, a Hann window halves that, a constant offset
/// yields `A * N` in the DC bin, and `divide_by_N` removes the `N`.
#[test]
#[cfg_attr(miri, ignore)] // runs forever + no real value add
fn test_magnitude_of_on_bin_sine() {
    const SAMPLING_RATE: u32 = 44100;
    const AMPLITUDE: f32 = 0.8;
    // relative tolerance; the FFT works with f32
    const TOLERANCE: f32 = 1e-3;

    let assert_close = |actual: f32, expected: f32, what: &str| {
        assert!(
            (actual - expected).abs() / expected < TOLERANCE,
            "{what}: expected {expected}, got {actual}"
        );
    };

    for n in [1024_usize, 4096, 16384] {
        let resolution = SAMPLING_RATE as f32 / n as f32;
        // a frequency that lies exactly on a bin, close to 1 kHz
        let frequency = (1000.0 / resolution).round() * resolution;
        let sine = (0..n)
            .map(|i| {
                let t = i as f32 / SAMPLING_RATE as f32;
                AMPLITUDE * (2.0 * PI * frequency * t).sin()
            })
            .collect::<Vec<f32>>();

        let spectrum =
            samples_fft_to_spectrum(&sine, SAMPLING_RATE, FrequencyLimit::All, None).unwrap();
        let (peak_fr, peak_val) = spectrum.max();
        assert!(
            (peak_fr.val() - frequency).abs() < resolution / 2.0,
            "peak must be at {frequency} Hz, got {peak_fr} Hz"
        );
        assert_close(
            peak_val.val(),
            AMPLITUDE * n as f32 / 2.0,
            "unscaled magnitude",
        );

        let spectrum = samples_fft_to_spectrum(
            &hann_window(&sine),
            SAMPLING_RATE,
            FrequencyLimit::All,
            None,
        )
        .unwrap();
        assert_close(
            spectrum.max().1.val(),
            AMPLITUDE * n as f32 / 4.0,
            "Hann-windowed magnitude",
        );

        let spectrum = samples_fft_to_spectrum(
            &sine,
            SAMPLING_RATE,
            FrequencyLimit::All,
            Some(&divide_by_N),
        )
        .unwrap();
        assert_close(
            spectrum.max().1.val(),
            AMPLITUDE / 2.0,
            "magnitude divided by N",
        );
    }

    // The DC bin holds A * N for a constant offset A, not A * N / 2.
    let n = 4096;
    let constant = vec![AMPLITUDE; n];
    let spectrum =
        samples_fft_to_spectrum(&constant, SAMPLING_RATE, FrequencyLimit::All, None).unwrap();
    assert_close(
        spectrum.dc_component().unwrap().val(),
        AMPLITUDE * n as f32,
        "DC component",
    );
}
