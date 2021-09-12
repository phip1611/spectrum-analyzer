/*
MIT License

Copyright (c) 2021 Philipp Schuster

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

//! Common utilities.

/// Maybe not the best name but this struct helps to calculate averages
/// when the elements are accumulated step by step.
#[derive(Debug)]
pub struct AverageBucket {
    sum: f32,
    n: u64,
}

impl AverageBucket {
    /// Constructs a new instance.
    pub const fn new() -> Self {
        Self { sum: 0.0, n: 0 }
    }

    /// Adds a new element.
    pub fn add(&mut self, num: f32) {
        debug_assert!(!num.is_nan());
        debug_assert!(num.is_finite());
        self.sum += num;
        self.n += 1;
    }

    /// Builds the average of all contained elements.
    pub fn avg(&mut self) -> f32 {
        if self.n == 0 {
            0.0
        } else {
            self.sum / self.n as f32
        }
    }

    /// Resets the internal state.
    pub fn reset(&mut self) {
        self.sum = 0.0;
        self.n = 0;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_average_sink() {
        let mut avg_sink = AverageBucket::new();
        assert_eq!(0.0, avg_sink.avg());
        avg_sink.add(1.0);
        assert_eq!(1.0, avg_sink.avg());
        avg_sink.add(2.0);
        avg_sink.add(3.0);
        assert_eq!(2.0, avg_sink.avg());
        avg_sink.reset();
        assert_eq!(0.0, avg_sink.avg());
    }
}
