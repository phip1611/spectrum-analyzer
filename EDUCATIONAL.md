## How to use FFT to get a frequency spectrum?

This library is full of additional and useful links and comments about how an FFT result
can be used to get a frequency spectrum. In this document I want to give a short introduction
where inside the code you can find specific things.

**TL;DR:** Although this crate has over 1000 lines of code, **the part which gets the frequency and
their values from the FFT is small and simple**. Most of the code is related to my convenient
abstraction over the FFT result including several getters, transform/scaling functions, and
tests.

**I don't explain how FFT works but how you use the result!**
If you want to understand that too:

- check out all links provided [at the end of README.md](/README.md)
- look into `lib.rs` (**probalby gives you 90 percent of the things you want to know**)
  and the comments over the FFT abstraction in `src/fft/mod.rs` and
  `src/fft/rustfft-complex/mod.rs`.


This is everything important you need. Everything inside
 `spectrum.rs` and the other files is just convenient stuff + tests for when you
want to use this crate in your program.
