# Changelog

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
### Other
- many internal improvements
- rust-toolchain.toml for build stability and reproducibility

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
