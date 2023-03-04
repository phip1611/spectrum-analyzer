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

use std::time::Instant;

use spectrum_analyzer::*;

/// Benchmark can be used to check how changes effect the performance.
/// Always execute with release flag!
fn main() {
    // create 2048 random samples
    let samples = (0..2048)
        .map(|_| rand::random::<i16>())
        .map(|x| x as f32)
        .collect::<Vec<_>>();
    let hann_window = windows::hann_window(&samples);

    let bench_res_without_scaling = bench_without_scaling(hann_window.clone());
    let bench_res_with_scaling = bench_with_scaling(hann_window);

    println!(
        "Bench without scaling: avg = {}us per Iteration",
        bench_res_without_scaling
    );
    println!(
        "Bench with    scaling: avg = {}us per Iteration",
        bench_res_with_scaling
    );
}

fn bench_without_scaling(samples: Vec<f32>) -> u64 {
    let fnc = move || samples_fft_to_spectrum(&samples, 44100, FrequencyLimit::All, None).unwrap();
    bench_fnc(Box::new(fnc))
}

fn bench_with_scaling(samples: Vec<f32>) -> u64 {
    let fnc = move || {
        samples_fft_to_spectrum(
            &samples,
            44100,
            FrequencyLimit::All,
            Some(&scaling::divide_by_N),
        )
        .unwrap()
    };
    bench_fnc(Box::new(fnc))
}

fn bench_fnc(fnc: Box<dyn Fn() -> FrequencySpectrum>) -> u64 {
    // warm-up
    for _ in 0..10 {
        let _ = fnc();
    }
    let now = Instant::now();
    let runs = 10000;
    for _ in 0..runs {
        let _ = fnc();
    }
    let duration = now.elapsed();
    (duration.as_micros() / runs) as u64
}
