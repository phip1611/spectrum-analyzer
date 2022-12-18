use criterion::{black_box, criterion_group, criterion_main, Criterion};
use spectrum_analyzer::{
    samples_fft_to_spectrum, scaling, windows, FrequencyLimit, FrequencySpectrum,
};

fn spectrum_without_scaling(samples: &[f32]) -> FrequencySpectrum {
    samples_fft_to_spectrum(samples, 44100, FrequencyLimit::All, None).unwrap()
}

fn spectrum_with_scaling(samples: &[f32]) -> FrequencySpectrum {
    samples_fft_to_spectrum(
        samples,
        44100,
        FrequencyLimit::All,
        Some(&scaling::divide_by_N_sqrt),
    )
    .unwrap()
}

fn spectrum_with_multiple_scaling(samples: &[f32]) -> FrequencySpectrum {
    let mut spectrum = spectrum_with_scaling(samples);

    let mut working_buffer = vec![(0.0.into(), 0.0.into()); spectrum.data().len()];

    spectrum
        .apply_scaling_fn(&scaling::divide_by_N_sqrt, &mut working_buffer)
        .unwrap();
    spectrum
        .apply_scaling_fn(&scaling::divide_by_N_sqrt, &mut working_buffer)
        .unwrap();
    spectrum
        .apply_scaling_fn(&scaling::divide_by_N_sqrt, &mut working_buffer)
        .unwrap();
    spectrum
}

fn criterion_benchmark(c: &mut Criterion) {
    // create 2048 random samples
    let samples = (0..2048)
        .map(|_| rand::random::<i16>())
        .map(|x| x as f32)
        .collect::<Vec<_>>();
    let hann_window = windows::hann_window(&samples);

    c.bench_function("spectrum without scaling", |b| {
        b.iter(|| spectrum_without_scaling(black_box(&hann_window)))
    });
    c.bench_function("spectrum with scaling", |b| {
        b.iter(|| spectrum_without_scaling(black_box(&hann_window)))
    });
    c.bench_function("spectrum with multiple scaling steps", |b| {
        b.iter(|| spectrum_with_multiple_scaling(black_box(&hann_window)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
