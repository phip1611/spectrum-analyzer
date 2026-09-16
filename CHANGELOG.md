# Changelog

## Unreleased (yet)

A major release with some breaking changes: the library no longer panics on
input it can check, and the types say more about the values they carry. Nothing
about the analysis itself changed, so the numbers you get out stay the same.

Documentation improved a lot.

### Migrating

- The two lookups return an `Option` now, so add a `?` or an `unwrap()`:
  `spectrum.freq_val_exact(1000.0)` and `freq_val_closest(1000.0)` give
  `None` for a frequency outside the spectrum instead of panicking.
- `Frequency` and `FrequencyValue` are different types now. Both compare and
  calculate with `f32` directly, so `assert!(value > 0.85)` works and most
  `val()` calls can go.
- Build a frequency limit with `FrequencyLimit::{min,max,range}` instead of
  the variants, which hold a `NonNegF32` now.
- Gone for good: `scaling::combined` (use a closure),
  `FrequencySpectrum::{median,to_map,to_mel_map,mel_val}` (use `data()` or
  `to_vec()`), `FrequencyLimit::{min,max}` as getters (use `maybe_min()` and
  `maybe_max()`), `FrequencySpectrum::new` and its `Default` implementation
  (a spectrum comes from `samples_fft_to_spectrum`).
- Errors moved around: `TooFewSamples` and `SamplesLengthNotAPowerOfTwo`
  became `InvalidLengthOfSamples`, and `InvalidSamplingRate` is new.

### Fewer panics

- **BREAKING** `FrequencySpectrum::{freq_val_exact,freq_val_closest}` return
  `None` for a frequency outside the spectrum, and for `NaN`, which slipped
  through the bounds check before
- **BREAKING** more than 32768 samples is an error instead of a panic
- **BREAKING** a sampling rate of zero is rejected with the new
  `SpectrumAnalyzerError::InvalidSamplingRate`, instead of producing a
  spectrum in which every frequency is `0 Hz`
- **BREAKING** removed `FrequencyLimit::{min,max}`, which panicked for
  variants without that bound
- **BREAKING** `FrequencySpectrum::new` is internal and the `Default`
  implementation is gone; the empty spectrum it produced made every getter
  panic

### Frequencies and values have their own types

- **BREAKING** a `Frequency` is a `NonNegF32` and a `FrequencyValue` a
  `FiniteF32`, so the two cannot be mixed up. The difference is real: a
  frequency is never negative, a value turns negative as soon as
  `scale_20_times_log10` touches it
- **BREAKING** `FrequencyLimit::{Min,Max,Range}` hold a `NonNegF32`, which
  makes a negative or non-regular limit impossible to build.
  `FrequencyLimitError::{NotARegularNumber,ValueBelowMinimum}` are gone with
  it
- **BREAKING** `FrequencySpectrum::frequency_resolution`,
  `scaling::SpectrumDataStats::{min,max,average}` and
  `FrequencyLimitError::{ValueAboveNyquist,InvalidRange}` carry those types
  as well. `SpectrumDataStats::n` stays an `f32`, since it is there to
  divide by

### A thinner spectrum

- **BREAKING** removed `scaling::combined`; it handed every function the
  statistics of the unscaled spectrum, so chaining two of them gave wrong
  results. A closure does the same job correctly
- **BREAKING** removed `FrequencySpectrum::median` and
  `SpectrumDataStats::median`; nothing in the library used them
- **BREAKING** removed `FrequencySpectrum::{to_map,to_mel_map}`; both used
  `u32` keys, so bins that shared a key silently overwrote each other
- **BREAKING** removed `FrequencySpectrum::mel_val`; such calculations are
  easy to do on `data()`, and the spectrum stays thin and unopinionated

### Fixed

- the Hamming and Blackman-Harris windows used the symmetric form (dividing
  by `N - 1`) while the Hann window used the periodic one (dividing by `N`).
  All of them use the periodic form now, which is the right one for FFT
  analysis. The coefficients change marginally and the coherent gains are
  exact
- `scale_20_times_log10` mapped `0.0` to `0 dB`, which ranked silence above
  every quieter bin. Values are clamped to `-100 dB` at minimum now
- the descriptions of `FrequencyLimit::Min` and `FrequencyLimit::Max` were
  swapped

### Performance

- spectrum creation no longer allocates and scans a working buffer for the
  median, which makes it 20-30% faster depending on the number of samples

### Documentation

- explained what the frequency values are and how they relate to the input
  signal, including the expected range of the samples
- added guidance on which window function and which scaling function to pick
- documented the coherent gain of each window, and the difference between the
  periodic and the symmetric form
- clarified `divide_by_N` against `divide_by_N_sqrt`, and the statistics a
  scaling function receives
- fixed the frequency resolution formula; it is `sample_rate / N`, not
  `sample_rate / (N / 2)`

## 1.9.0 (2026-09-05)

Combined, `samples_fft_to_spectrum` got roughly 2-3x faster for 2048 samples
and ~2.3x faster for 16384 samples.

- perf: spectrum statistics (min/max/median/average) are computed in `O(n)`
  instead of sorting the data (up to ~50% faster spectrum computation)
- perf: a `FrequencyLimit` now stops the bin iteration at its bounds instead
  of testing every bin up to the Nyquist frequency (~30% faster for narrow
  frequency limits)
- perf: the spectrum vector is allocated up front instead of growing through
  several re-allocations (~25-40% faster spectrum computation)
- perf: input validation scans the samples once instead of twice (~5% faster);
  cheap checks (power-of-two length, frequency limit) now run before the scan,
  which may change the returned error variant for inputs with multiple
  problems

## 1.8.0 (2026-07-02)

- **BREAKING** MSRV is now `1.85.1` and the crate uses the 2024 edition
- fixed Blackman-Harris window functions
- fixed Hamming window coefficients
- fixed median calculation for spectra with an odd number of bins
- fixed `Frequency` and `FrequencyValue` accepting NaN and infinity in release builds
- fixed narrow frequency limits returning an internal panic
- fixed the scaled benchmark running the unscaled implementation
- fixed the MP3 example using the 4-term Blackman-Harris window for its 7-term output

## 1.7.0 (2025-05-11)
- **BREAKING** MSRV is now `1.81.0`
- Error types now implement the `Error` trait

## 1.6.0 (2024-12-17)
- dependency updates
- MSRV bump but only for the tests and examples, not library users
- Added FFT buffer size of 32768
- Optimized implementation, resulting in less unnecessary copying of data
- Removed excessive stack usage for large input data

## 1.5.0 (2023-09-21)
- fixed the build by updating the dependencies
- apart from that, no changes happened
- **BREAKING** MSRV is now `1.63.0`
- internal code improvements

## 1.4.0 (2023-03-04)
- dropped all optional FFT features (`microfft-complex`, `microfft-real`,
  `rustfft-complex`) and made `microfft::real` the default FFT implementation.
  This is breaking but only for a small percentage of users. There was no
  benefit from that feature and `microfft::real`, the default, was already the
  fastest for no_std as well as std targets.
- dependency updates

## 1.3.0 (2023-03-04)
- MSRV is now `1.63.0`
- `FrequencySpectrum::apply_scaling_fn` now requires a reference to `&mut self`:
  This is breaking but only for a small percentage of users. Performance is
  slightly improved as less heap allocations are required.
- `FrequencySpectrum` is now `Send` and interior mutability is dropped:
  You can wrap the struct in a `Mutex` or similar types now!
- `FrequencySpectrum::to_map` doesn't  has the `scaling_fn` parameter anymore.
  This is breaking but only for a small percentage of users.
- `FrequencySpectrum::to_mel_map` added for getting the spectrum in the
  [mel](https://en.wikipedia.org/wiki/Mel_scale) scale.
- small internal code quality and performance improvements

## 1.2.6 (2022-07-20)
- fixed wrong scaling in `scaling::divide_by_N_sqrt` (<https://github.com/phip1611/spectrum-analyzer/issues/41>)

## 1.2.5 (2022-06-19)
- dependency update
- number of sample can now also be 8192 and 16384 (when the feature `microfft` is used)

## 1.2.4 (2022-04-06)
- added scaling fn `divide_by_N_sqrt` which is the recommended
  normalization function by the `rustfft` documentation (but generally applicable)

## 1.2.2 / 1.2.3 (2021-11-26)
- typos and small fixes

## 1.2.1 (2021-11-15)
- removed `FrequencySpectrum::to_log_spectrum` (was useless; my understanding was wrong)
- added example `live-visualization` that shows how you could visualize a spectrum in real-time \
  ![Example visualization of real-time audio + spectrum analysis](res/live_demo_spectrum_green_day_holiday.gif "Example visualization of real-time audio + spectrum analysis")
- removed example `live-spectrum-visualization` (was really bad)

## 1.2.0 (2021-11-10)
- MSRV is now 1.56.1
- Rust edition 2021

## 1.1.2 (2021-09-14)
- example `live-spectrum-visualization` requires Rust stable `1.55.0` because
  the `ringbuffer`-dependency requires it
- the rest of the crate should still build with the current MSRV `1.52.1`
- if you run tests, you need `1.55.0`!

## 1.1.1
- bugfix of a bug since v1.0.0

## 1.1.0
- added `scaling::combined` which helps you to combine several scaling functions
- added `FrequencySpectrum::to_log_spectrum` which gives you a more usable spectrum
  when you analyze music for example.
  - this is not optimal yet :(
  - Needs code contributions.. doesn't look as nice and convenient as other implementations.
    I don't really know what to do here.

## v1.0.0
### Breaking Changes
- The library returns now a result and will no longer panic
  under certain circumstances.
- removed `per_element_scaling_fn` in favor of
  `complex_scaling_fn`, which is now just called
  `scaling_fn` - the old behaviour was more confusing than
  beneficial
- renamed `ComplexSpectrumScalingFunction` to `SpectrumScalingFunction` and
  moved it into the `scaling`-module
- MSRV is now `1.52.1` stable
### Other
- many internal improvements
- rust-toolchain.toml for build stability and reproducibility
- really minor performance improvements (~5 %)
- added example `live-spectrum-visualization.rs`
  (still not perfect, but works)
- added example `bench.rs`

## v0.5.1
- Feature "rustfft-complex" uses "rustfft"-crate at version 6 which is faster/more optimized (~25%).
- improved CI
- README update

## v0.5.0
This crate now uses `microfft::real` as default FFT implementation. It is by far the fastest implementation
and there are no disadvantages, despite (with `microfft` version 0.4.0) the maximum FFT size is 4096. If you
need bigger FFT sizes, use feature `rustfft-complex`.

This crate now works in `no_std`-environments by default.

## v0.4.5
Added MIT to file headers where it was missing.

## v0.4.4
Fixed wrong usage of `microfft::real` + bumped version of `microfft` to `0.4.0`.
**Currently it seems like with this implementation you only can get
the frequencies zero to `sampling_rate/4`, i.e. half of Nyquist frequency!**
I found out so by plotting the values. Wait until
https://gitlab.com/ra_kete/microfft-rs/-/issues/9 gets resolved.

## v0.4.3
README fix.

## v0.4.2
Typo in README.md code example.

## v0.4.1
Typo in README.md.

## v0.4.0
- MSRV is now Rust 1.51 (sorry but that's the only way I can make it `no_std`-compatible)
- This crate is now really `no_std`
- you can choose between three FFT implementations at compile time via Cargo features.
  The new default feature is `rustfft-complex` (which is still `std`) but there are also
  the two new `no_std`-compatible targets `microfft-complex` (more accurate, like rustfft)
  and `microfft-real` (faster, less accurate)
- several small improvements and fixes, see: https://github.com/phip1611/spectrum-analyzer/milestone/3?closed=1

## v0.3.0
- `FrequencySpectrum::min()` and `FrequencySpectrum::max()`
   now return tuples/pairs (issue #6)
- `FrequencySpectrum` now has convenient methods to get
   the value of a desired frequency from the underlying vector
   of FFT results (issue #8)
- `FrequencySpectrum` now has a field + getter for `frequency_resolution`
  (issue #5)

For all issues see: https://github.com/phip1611/spectrum-analyzer/milestone/2
