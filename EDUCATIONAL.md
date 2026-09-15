## How to use FFT to get a frequency spectrum?

**This document does not explain how FFT works, but how its result is used.**
It tells you where in the code to look.

**TL;DR:** Although this crate has over 1000 lines of code, **the part that
turns the FFT result into frequencies and their values is small and simple**.
The rest is the convenient abstraction around it: getters, scaling functions,
and tests.

Where to look:

- `src/lib.rs` (**probably gives you 90 percent of the things you want to
  know**), especially `fft_result_to_spectrum`, which calculates the
  frequency of each FFT result index and its magnitude
- the comments over the FFT abstraction in `src/fft.rs`
- the documentation of `samples_fft_to_spectrum` for what the resulting
  values mean

If you want to understand FFT itself, follow the links at the end of
[README.md](/README.md).

`src/spectrum.rs` and the other files are convenience and tests. You do not
need them to understand the idea.
