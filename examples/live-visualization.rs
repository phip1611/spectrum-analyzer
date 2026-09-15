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
use audio_visualizer::live::{LiveVisualizer, Transform};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};

mod common;

/// Example that creates a live visualization of the frequency spectrum of
/// realtime audio data
/// **Execute this with `--release` for better performance!**.
fn main() {
    // Spectrum of the previous frame; used to smoothen the visualization:
    // each frequency value decays over time and is replaced when the new
    // value is higher.
    let mut smoothed: Vec<(f64, f64)> = vec![];

    let to_spectrum = move |samples: &[f32], sample_rate: f32| {
        // spectrum analysis of the latest ~46ms (at 44.1kHz)
        let latest = &samples[samples.len() - 2048..];
        let hann_window = hann_window(latest);
        let spectrum = samples_fft_to_spectrum(
            &hann_window,
            sample_rate as u32,
            FrequencyLimit::All,
            Some(&divide_by_N),
        )
        .unwrap();

        let current = spectrum
            .data()
            .iter()
            .map(|(f, v)| (f.val() as f64, (v.val() * 5000.0) as f64))
            .collect::<Vec<_>>();
        if smoothed.len() != current.len() {
            smoothed = current;
        } else {
            for ((_, old), (_, new)) in smoothed.iter_mut().zip(&current) {
                *old = (*old * 0.84).max(*new);
            }
        }
        smoothed.clone()
    };

    let input = common::select_input();
    LiveVisualizer::new(Transform::points(to_spectrum))
        .title("Live Spectrum View")
        .axis_labels("frequency (Hz)", "magnitude")
        .x_range(0.0..22050.0)
        .y_range(0.0..500.0)
        .input(input)
        .open()
        .unwrap();
}
