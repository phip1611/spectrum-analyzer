# Changelog

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
