# Rust: library for frequency spectrum analysis using FFT
A simple and fast `no_std` library to get the frequency spectrum of a digital signal (e.g. audio) using FFT.
It follows the KISS principle and consists of simple building blocks/optional features. In short, this is 
a convenient wrapper around several FFT implementations which you can choose from during compilation time
via using Cargo features. As of version 0.4.0 this uses "microfft"-crate.

**I'm not an expert on digital signal processing. Code contributions are highly welcome! 🙂**

**The MSRV (minimum supported Rust version) is 1.51 Stable because this Crate needs the "resolver" feature of Cargo.**

## How to use
Most tips and comments are located inside the code, so please check out the repository on
Github! Anyway, the most basic usage looks like this:

### FFT implementation as compile time configuration via Cargo features
This crate offers two FFT implementations from the `microfft`-crate. One feature is called `microfft-complex` and uses 
a "typical/regular" complex FFT which results in results of higher accuracy whereas `microfft-real` doesn't need 
complex numbers which is faster but less accurate. It depends on your use case. Plot the results to see the differences.
On today's hardware, the complex version should always be fine and fast enough.

### Cargo.toml
```Cargo.toml
# fixes build problems of wrong feature resolution in microfft crate, see
# https://gitlab.com/ra_kete/microfft-rs/-/merge_requests/11
# this requires Rust Stable 1.51
resolver = "2"

# by default feature "microfft-complex" is used
spectrum-analyzer = { version = "0.3.0" }
# or
spectrum-analyzer = { version = "0.3.0", default-features = false, features = "microfft-real" }
```

### your_binary.rs
```rust
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use spectrum_analyzer::windows::hann_window;

fn main() {
    // This lib also works in `no_std` environments!
    let samples: &[f32] = get_samples(); // TODO you need to implement the samples source
    // apply hann window for smoothing; length must be a power of 2 for the FFT
    let hann_window = hann_window(&samples[0..4096]);
    // calc spectrum
    let spectrum_hann_window = samples_fft_to_spectrum(
        // (windowed) samples
        &hann_window,
        // sample rate
        44100,
        // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
        FrequencyLimit::All,
        // optional per element scaling function, e.g. `20 * log10(x)`; see doc comments
        None,
        // optional total scaling at the end; see doc comments
        None,
    );

    for (fr, fr_val) in spectrum_hamming_window.raw_data().iter() {
        println!("{}Hz => {}", fr, fr_val)
    }
}
```

## Scaling the frequency values/amplitudes
As already mentioned, there are lots of comments in the code. Short story is:
Type `ComplexSpectrumScalingFunction` can do anything whereas `BasicSpectrumScalingFunction`
is easier to write, especially for Rust beginners.

## Performance
*Measurements taken on i7-8650U @ 3 Ghz (Single-Core) with optimized build*


| Operation                                     | Time   |
| --------------------------------------------- | ------:|
| Hann Window with 4096 samples                 | ≈70µs  |
| Hamming Window with 4096 samples              | ≈10µs  |
| Hann Window with 16384 samples                | ≈175µs |
| Hamming Window with 16384 samples             | ≈44µs  |
| FFT to spectrum with 4096 samples @ 44100Hz   | ≈240µs |
| FFT to spectrum with 16384 samples @ 44100Hz  | ≈740µs |

## Example visualization
In the following example you can see a basic visualization of frequencies `0 to 4000Hz` for 
a layered signal of sine waves of `50`, `1000`, and `3777Hz` @ `41000Hz` sample rate. The peaks for the 
given frequencies are clearly visible. Each calculation was done with `2048` samples, i.e. ≈46ms.

**The noise (wrong peaks) also comes from clipping of the added sine waves!**

### Spectrum without window function on samples
Peaks (50, 1000, 3777 Hz) are clearly visible but also some noise.
![Visualization of spectrum 0-4000Hz of layered sine signal (50, 1000, 3777 Hz)) with no window function.](spectrum_sine_waves_50_1000_3777hz--no-window.png "Peaks (50, 1000, 3777 Hz) are clearly visible but also some noise.")

### Hann window function on samples before FFT
Peaks (50, 1000, 3777 Hz) are clearly visible and Hann window reduces noise a little bit. Because this example has few noise, you don't see much difference.
![Visualization of spectrum 0-4000Hz of layered sine signal (50, 1000, 3777 Hz)) with Hann window function.](spectrum_sine_waves_50_1000_3777hz--no-window.png "Peaks (50, 1000, 3777 Hz) are clearly visible and Hann window reduces noise a little bit. Because this example has few noise, you don't see much difference.")

### Hamming window function on samples before FFT
Peaks (50, 1000, 3777 Hz) are clearly visible and Hamming window reduces noise a little bit. Because this example has few noise, you don't see much difference.
![Visualization of spectrum 0-4000Hz of layered sine signal (50, 1000, 3777 Hz)) with Hamming window function.](spectrum_sine_waves_50_1000_3777hz--no-window.png "Peaks (50, 1000, 3777 Hz) are clearly visible and Hamming window reduces noise a little bit. Because this example has few noise, you don't see much difference.")

## Trivia / FAQ
### Why f64 and no f32?
I tested f64 but the additional accuracy doesn't pay out the ~40% calculation overhead (on x86_64).
### What can I do against the noise?
Apply a window function, like Hann window or Hamming window. But I'm not an expert on this.

## Good resources with more information
- Interpreting FFT Results: https://www.gaussianwaves.com/2015/11/interpreting-fft-results-complex-dft-frequency-bins-and-fftshift/
- FFT basic concepts: https://www.youtube.com/watch?v=z7X6jgFnB6Y
- „The Fundamentals of FFT-Based Signal Analysis and Measurement“ https://www.sjsu.edu/people/burford.furman/docs/me120/FFT_tutorial_NI.pdf
- Fast Fourier Transforms (FFTs) and Windowing: https://www.youtube.com/watch?v=dCeHOf4cJE0

Also check out my blog post! https://phip1611.de/2021/03/programmierung-und-skripte/frequency-spectrum-analysis-with-fft-in-rust/

### Real vs Complex FFT: Accuracy
The two FFT implementations have different advantages and your decision for one of 
them is a tradeoff between accuracy and computation time. The following two 
screenshots plot a spectrum obtained by real FFT respectively complex FFT.
The complex FFT result is much smoother.
#### Real FFT (less accuracy)
![Spectrum obtained using real FFT: 60 Hz and 100 Hz sine wave signal](real-fft-60_and_100_hz.png "Spectrum obtained using real FFT: 60 Hz and 100 Hz sine wave signal")
#### Real FFT (more accuracy)
![Spectrum obtained using complex FFT: 60 Hz and 100 Hz sine wave signal](complex-fft-60_and_100_hz.png "Spectrum obtained complex real FFT: 60 Hz and 100 Hz sine wave signal")

