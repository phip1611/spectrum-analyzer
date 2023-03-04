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
//! Module for the struct [`FrequencySpectrum`].

use self::math::*;
use crate::error::SpectrumAnalyzerError;
use crate::frequency::{Frequency, FrequencyValue};
use crate::scaling::{SpectrumDataStats, SpectrumScalingFunction};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Convenient wrapper around the processed FFT result which describes each
/// frequency and its value/amplitude from the analyzed samples. It only
/// contains the frequencies that were desired, e.g., specified via
/// [`crate::limit::FrequencyLimit`] when [`crate::samples_fft_to_spectrum`]
/// was called.
///
/// This means, the spectrum can cover all data from the DC component (0Hz) to
/// the Nyquist frequency.
///
/// All results are related to the sampling rate provided to the library
/// function which creates objects of this struct!
///
/// This struct can be shared across thread boundaries.
#[derive(Debug, Default)]
pub struct FrequencySpectrum {
    /// All (Frequency, FrequencyValue) data pairs sorted by lowest frequency
    /// to the highest frequency.Vector is sorted from lowest
    /// frequency to highest and data is normalized/scaled
    /// according to all applied scaling functions.
    data: Vec<(Frequency, FrequencyValue)>,
    /// Frequency resolution of the examined samples in Hertz,
    /// i.e the frequency steps between elements in the vector
    /// inside field [`Self::data`].
    frequency_resolution: f32,
    /// Number of samples that were analyzed. Might be bigger than the length
    /// of `data`, if the spectrum was created with a [`crate::limit::FrequencyLimit`] .
    samples_len: u32,
    /// Average value of frequency value/magnitude/amplitude
    /// corresponding to data in [`FrequencySpectrum::data`].
    average: FrequencyValue,
    /// Median value of frequency value/magnitude/amplitude
    /// corresponding to data in [`FrequencySpectrum::data`].
    median: FrequencyValue,
    /// Pair of (frequency, frequency value/magnitude/amplitude) where
    /// frequency value is **minimal** inside the spectrum.
    /// Corresponding to data in [`FrequencySpectrum::data`].
    min: (Frequency, FrequencyValue),
    /// Pair of (frequency, frequency value/magnitude/amplitude) where
    /// frequency value is **maximum** inside the spectrum.
    /// Corresponding to data in [`FrequencySpectrum::data`].
    max: (Frequency, FrequencyValue),
}

impl FrequencySpectrum {
    /// Creates a new object. Calculates several metrics from the data
    /// in the given vector.
    ///
    /// ## Parameters
    /// * `data` Vector with all ([`Frequency`], [`FrequencyValue`])-tuples
    /// * `frequency_resolution` Resolution in Hertz. This equals to
    ///                          `data[1].0 - data[0].0`.
    /// * `samples_len` Number of samples. Might be bigger than `data.len()`
    ///                 if the spectrum is obtained with a frequency limit.
    /// * `working_buffer` Mutable buffer with the same length as `data`
    ///                    required to calculate certain metrics.
    #[inline]
    #[must_use]
    pub fn new(
        data: Vec<(Frequency, FrequencyValue)>,
        frequency_resolution: f32,
        samples_len: u32,
        working_buffer: &mut [(Frequency, FrequencyValue)],
    ) -> Self {
        debug_assert!(
            data.len() >= 2,
            "Input data of length={} for spectrum makes no sense!",
            data.len()
        );

        let mut obj = Self {
            data,
            frequency_resolution,
            samples_len,
            // default/placeholder values
            average: FrequencyValue::from(-1.0),
            median: FrequencyValue::from(-1.0),
            min: (Frequency::from(-1.0), FrequencyValue::from(-1.0)),
            max: (Frequency::from(-1.0), FrequencyValue::from(-1.0)),
        };

        // Important to call this once initially.
        obj.calc_statistics(working_buffer);
        obj
    }

    /// Applies the function `scaling_fn` to each element and updates several
    /// metrics about the spectrum, such as `min` and `max`, afterwards
    /// accordingly. It ensures that no value is `NaN` or `Infinity`
    /// (regarding IEEE-754) after `scaling_fn` was applied. Otherwise,
    /// `SpectrumAnalyzerError::ScalingError` is returned.
    ///
    /// ## Parameters
    /// * `scaling_fn` See [`crate::scaling::SpectrumScalingFunction`].
    #[inline]
    pub fn apply_scaling_fn(
        &mut self,
        scaling_fn: &SpectrumScalingFunction,
        working_buffer: &mut [(Frequency, FrequencyValue)],
    ) -> Result<(), SpectrumAnalyzerError> {
        // This represents statistics about the spectrum in its current state
        // which a scaling function may use to scale values.
        //
        // On the first invocation of this function, these values represent the
        // statistics for the unscaled, hence initial, spectrum.
        let stats = SpectrumDataStats {
            min: self.min.1.val(),
            max: self.max.1.val(),
            average: self.average.val(),
            median: self.median.val(),
            // attention! not necessarily `data.len()`!
            n: self.samples_len as f32,
        };

        // Iterate over the whole spectrum and scale each frequency value.
        // I use a regular for loop instead of for_each(), so that I can
        // early return a result here
        for (_fr, fr_val) in &mut self.data {
            // scale value
            let scaled_val: f32 = scaling_fn(fr_val.val(), &stats);

            // sanity check
            if scaled_val.is_nan() || scaled_val.is_infinite() {
                return Err(SpectrumAnalyzerError::ScalingError(
                    fr_val.val(),
                    scaled_val,
                ));
            }

            // Update value in spectrum
            *fr_val = scaled_val.into()
        }

        self.calc_statistics(working_buffer);
        Ok(())
    }

    /// Returns the average frequency value of the spectrum.
    #[inline]
    #[must_use]
    pub const fn average(&self) -> FrequencyValue {
        self.average
    }

    /// Returns the median frequency value of the spectrum.
    #[inline]
    #[must_use]
    pub const fn median(&self) -> FrequencyValue {
        self.median
    }

    /// Returns the maximum (frequency, frequency value)-pair of the spectrum
    /// **regarding the frequency value**.
    #[inline]
    #[must_use]
    pub const fn max(&self) -> (Frequency, FrequencyValue) {
        self.max
    }

    /// Returns the minimum (frequency, frequency value)-pair of the spectrum
    /// **regarding the frequency value**.
    #[inline]
    #[must_use]
    pub const fn min(&self) -> (Frequency, FrequencyValue) {
        self.min
    }

    /// Returns [`FrequencySpectrum::max().1`] - [`FrequencySpectrum::min().1`],
    /// i.e. the range of the frequency values (not the frequencies itself,
    /// but their amplitudes/values).
    #[inline]
    #[must_use]
    pub fn range(&self) -> FrequencyValue {
        self.max().1 - self.min().1
    }

    /// Returns the underlying data.
    #[inline]
    #[must_use]
    pub fn data(&self) -> &[(Frequency, FrequencyValue)] {
        &self.data
    }

    /// Returns the frequency resolution of this spectrum.
    #[inline]
    #[must_use]
    pub const fn frequency_resolution(&self) -> f32 {
        self.frequency_resolution
    }

    /// Returns the number of samples used to obtain this spectrum.
    #[inline]
    #[must_use]
    pub const fn samples_len(&self) -> u32 {
        self.samples_len
    }

    /// Getter for the highest frequency that is captured inside this spectrum.
    /// Shortcut for `spectrum.data()[spectrum.data().len() - 1].0`.
    /// This corresponds to the [`crate::limit::FrequencyLimit`] of the spectrum.
    ///
    /// This method could return the Nyquist frequency, if there was no Frequency
    /// limit while obtaining the spectrum.
    #[inline]
    #[must_use]
    pub fn max_fr(&self) -> Frequency {
        self.data[self.data.len() - 1].0
    }

    /// Getter for the lowest frequency that is captured inside this spectrum.
    /// Shortcut for `spectrum.data()[0].0`.
    /// This corresponds to the [`crate::limit::FrequencyLimit`] of the spectrum.
    ///
    /// This method could return the DC component, see [`Self::dc_component`].
    #[inline]
    #[must_use]
    pub fn min_fr(&self) -> Frequency {
        self.data[0].0
    }

    /// Returns the *DC Component* or also called *DC bias* which corresponds
    /// to the FFT result at index 0 which corresponds to `0Hz`. This is only
    /// present if the frequencies were not limited to for example `100 <= f <= 10000`
    /// when the libraries main function was called.
    ///
    /// More information:
    /// <https://dsp.stackexchange.com/questions/12972/discrete-fourier-transform-what-is-the-dc-term-really>
    ///
    /// Excerpt:
    /// *As far as practical applications go, the DC or 0 Hz term is not particularly useful.
    /// In many cases it will be close to zero, as most signal processing applications will
    /// tend to filter out any DC component at the analogue level. In cases where you might
    /// be interested it can be calculated directly as an average in the usual way, without
    /// resorting to a DFT/FFT.* - Paul R.
    #[inline]
    #[must_use]
    pub fn dc_component(&self) -> Option<FrequencyValue> {
        let (maybe_dc_component, dc_value) = &self.data[0];
        if maybe_dc_component.val() == 0.0 {
            Some(*dc_value)
        } else {
            None
        }
    }

    /// Returns the value of the given frequency from the spectrum either exactly or approximated.
    /// If `search_fr` is not exactly given in the spectrum, i.e. due to the
    /// [`Self::frequency_resolution`], this function takes the two closest
    /// neighbors/points (A, B), put a linear function through them and calculates
    /// the point C in the middle. This is done by the private function
    /// `calculate_y_coord_between_points`.
    ///
    /// ## Panics
    /// If parameter `search_fr` (frequency) is below the lowest or the maximum
    /// frequency, this function panics! This is because the user provide
    /// the min/max frequency when the spectrum is created and knows about it.
    /// This is similar to an intended "out of bounds"-access.
    ///
    /// ## Parameters
    /// - `search_fr` The frequency of that you want the amplitude/value in the spectrum.
    ///
    /// ## Return
    /// Either exact value of approximated value, determined by [`Self::frequency_resolution`].
    #[inline]
    #[must_use]
    pub fn freq_val_exact(&self, search_fr: f32) -> FrequencyValue {
        // lowest frequency in the spectrum
        let (min_fr, min_fr_val) = self.data[0];
        // highest frequency in the spectrum
        let (max_fr, max_fr_val) = self.data[self.data.len() - 1];

        // https://docs.rs/float-cmp/0.8.0/float_cmp/
        let equals_min_fr = float_cmp::approx_eq!(f32, min_fr.val(), search_fr, ulps = 3);
        let equals_max_fr = float_cmp::approx_eq!(f32, max_fr.val(), search_fr, ulps = 3);

        // Fast return if possible
        if equals_min_fr {
            return min_fr_val;
        }
        if equals_max_fr {
            return max_fr_val;
        }
        // bounds check
        if search_fr < min_fr.val() || search_fr > max_fr.val() {
            panic!(
                "Frequency {}Hz is out of bounds [{}; {}]!",
                search_fr,
                min_fr.val(),
                max_fr.val()
            );
        }

        // We search for Point C (x=search_fr, y=???) between Point A and Point B iteratively.
        // Point B is always the successor of A.

        for two_points in self.data.iter().as_slice().windows(2) {
            let point_a = two_points[0];
            let point_b = two_points[1];
            let point_a_x = point_a.0.val();
            let point_a_y = point_a.1;
            let point_b_x = point_b.0.val();
            let point_b_y = point_b.1.val();

            // check if we are in the correct window; we are in the correct window
            // iff point_a_x <= search_fr <= point_b_x
            if search_fr > point_b_x {
                continue;
            }

            return if float_cmp::approx_eq!(f32, point_a_x, search_fr, ulps = 3) {
                // directly return if possible
                point_a_y
            } else {
                calculate_y_coord_between_points(
                    (point_a_x, point_a_y.val()),
                    (point_b_x, point_b_y),
                    search_fr,
                )
                .into()
            };
        }

        panic!("Here be dragons");
    }

    /// Returns the frequency closest to parameter `search_fr` in the spectrum. For example
    /// if the spectrum looks like this:
    /// ```text
    /// Vector:    [0]      [1]      [2]      [3]
    /// Frequency  100 Hz   200 Hz   300 Hz   400 Hz
    /// Fr Value   0.0      1.0      0.5      0.1
    /// ```
    /// then `get_frequency_value_closest(320)` will return `(300.0, 0.5)`.
    ///
    /// ## Panics
    /// If parameter `search_fre` (frequency) is below the lowest or the maximum
    /// frequency, this function panics!
    ///
    /// ## Parameters
    /// - `search_fr` The frequency of that you want the amplitude/value in the spectrum.
    ///
    /// ## Return
    /// Closest matching point in spectrum, determined by [`Self::frequency_resolution`].
    #[inline]
    #[must_use]
    pub fn freq_val_closest(&self, search_fr: f32) -> (Frequency, FrequencyValue) {
        // lowest frequency in the spectrum
        let (min_fr, min_fr_val) = self.data[0];
        // highest frequency in the spectrum
        let (max_fr, max_fr_val) = self.data[self.data.len() - 1];

        // https://docs.rs/float-cmp/0.8.0/float_cmp/
        let equals_min_fr = float_cmp::approx_eq!(f32, min_fr.val(), search_fr, ulps = 3);
        let equals_max_fr = float_cmp::approx_eq!(f32, max_fr.val(), search_fr, ulps = 3);

        // Fast return if possible
        if equals_min_fr {
            return (min_fr, min_fr_val);
        }
        if equals_max_fr {
            return (max_fr, max_fr_val);
        }

        // bounds check
        if search_fr < min_fr.val() || search_fr > max_fr.val() {
            panic!(
                "Frequency {}Hz is out of bounds [{}; {}]!",
                search_fr,
                min_fr.val(),
                max_fr.val()
            );
        }

        for two_points in self.data.iter().as_slice().windows(2) {
            let point_a = two_points[0];
            let point_b = two_points[1];
            let point_a_x = point_a.0;
            let point_a_y = point_a.1;
            let point_b_x = point_b.0;
            let point_b_y = point_b.1;

            // check if we are in the correct window; we are in the correct window
            // iff point_a_x <= search_fr <= point_b_x
            if search_fr > point_b_x.val() {
                continue;
            }

            return if float_cmp::approx_eq!(f32, point_a_x.val(), search_fr, ulps = 3) {
                // directly return if possible
                (point_a_x, point_a_y)
            } else {
                // absolute difference
                let delta_to_a = search_fr - point_a_x.val();
                // let delta_to_b = point_b_x.val() - search_fr;
                if delta_to_a / self.frequency_resolution < 0.5 {
                    (point_a_x, point_a_y)
                } else {
                    (point_b_x, point_b_y)
                }
            };
        }

        panic!("Here be dragons");
    }

    /// Wrapper around [`Self::freq_val_exact`] that consumes [mel].
    ///
    /// [mel]: https://en.wikipedia.org/wiki/Mel_scale
    #[inline]
    #[must_use]
    pub fn mel_val(&self, mel_val: f32) -> FrequencyValue {
        let hz = mel_to_hertz(mel_val);
        self.freq_val_exact(hz)
    }

    /// Returns a [`BTreeMap`] with all value pairs. The key is of type [`u32`]
    /// because [`f32`] is not [`Ord`].
    #[inline]
    #[must_use]
    pub fn to_map(&self) -> BTreeMap<u32, f32> {
        self.data
            .iter()
            .map(|(fr, fr_val)| (fr.val() as u32, fr_val.val()))
            .collect()
    }

    /// Like [`Self::to_map`] but converts the frequency (x-axis) to [mels]. The
    /// resulting map contains more results in a higher density the higher the
    /// mel value gets. This comes from the logarithmic transformation from
    /// hertz to mels.
    ///
    /// [mels]: https://en.wikipedia.org/wiki/Mel_scale
    #[inline]
    #[must_use]
    pub fn to_mel_map(&self) -> BTreeMap<u32, f32> {
        self.data
            .iter()
            .map(|(fr, fr_val)| (hertz_to_mel(fr.val()) as u32, fr_val.val()))
            .collect()
    }

    /// Calculates the `min`, `max`, `median`, and `average` of the frequency values/magnitudes/
    /// amplitudes.
    ///
    /// To do so, it needs to create a sorted copy of the data.
    #[inline]
    fn calc_statistics(&mut self, working_buffer: &mut [(Frequency, FrequencyValue)]) {
        // We create a copy with all data from `self.data` but we sort it by the
        // frequency value and not the frequency. This way, we can easily find the
        // median.

        let data_sorted_by_val = {
            assert_eq!(
                self.data.len(),
                working_buffer.len(),
                "The working buffer must have the same length as `self.data`!"
            );

            for (i, pair) in self.data.iter().enumerate() {
                working_buffer[i] = *pair;
            }
            working_buffer.sort_by(|(_l_fr, l_fr_val), (_r_fr, r_fr_val)| {
                // compare by frequency value, from min to max
                l_fr_val.cmp(r_fr_val)
            });

            working_buffer
        };

        // sum of all frequency values
        let sum: f32 = data_sorted_by_val
            .iter()
            .map(|fr_val| fr_val.1.val())
            .fold(0.0, |a, b| a + b);

        // average of all frequency values
        let avg = sum / data_sorted_by_val.len() as f32;
        let average: FrequencyValue = avg.into();

        // median of all frequency values
        let median = {
            // we assume that data_sorted_by_val.length() is always even, because
            // it must be a power of 2 (for FFT)
            let a = data_sorted_by_val[data_sorted_by_val.len() / 2 - 1].1;
            let b = data_sorted_by_val[data_sorted_by_val.len() / 2].1;
            (a + b) / 2.0.into()
        };

        // Because we sorted the vector from lowest to highest value, the
        // following lines are correct, i.e., we get min/max value with
        // the corresponding frequency.
        let min = data_sorted_by_val[0];
        let max = data_sorted_by_val[data_sorted_by_val.len() - 1];

        // check that I get the comparison right (and not from max to min)
        debug_assert!(min.1 <= max.1, "min must be <= max");

        self.min = min;
        self.max = max;
        self.average = average;
        self.median = median;
    }
}

/*impl FromIterator<(Frequency, FrequencyValue)> for FrequencySpectrum {

    #[inline]
    fn from_iter<T: IntoIterator<Item=(Frequency, FrequencyValue)>>(iter: T) -> Self {
        // 1024 is just a guess: most likely 2048 is a common FFT length,
        // i.e. 1024 results for the frequency spectrum.
        let mut vec = Vec::with_capacity(1024);
        for (fr, val) in iter {
            vec.push((fr, val))
        }

        FrequencySpectrum::new(vec)
    }
}*/

mod math {
    // use super::*;

    /// Calculates the y coordinate of Point C between two given points A and B
    /// if the x-coordinate of C is known. It does that by putting a linear function
    /// through the two given points.
    ///
    /// ## Parameters
    /// - `(x1, y1)` x and y of point A
    /// - `(x2, y2)` x and y of point B
    /// - `x_coord` x coordinate of searched point C
    ///
    /// ## Return Value
    /// y coordinate of searched point C
    #[inline]
    pub fn calculate_y_coord_between_points(
        (x1, y1): (f32, f32),
        (x2, y2): (f32, f32),
        x_coord: f32,
    ) -> f32 {
        // e.g. Points (100, 1.0) and (200, 0.0)
        // y=f(x)=-0.01x + c
        // 1.0 = f(100) = -0.01x + c
        // c = 1.0 + 0.01*100 = 2.0
        // y=f(180)=-0.01*180 + 2.0

        // gradient, anstieg
        let slope = (y2 - y1) / (x2 - x1);
        // calculate c in y=f(x)=slope * x + c
        let c = y1 - slope * x1;

        slope * x_coord + c
    }

    /// Converts hertz to [mel](https://en.wikipedia.org/wiki/Mel_scale).
    pub fn hertz_to_mel(hz: f32) -> f32 {
        assert!(hz >= 0.0);
        2595.0 * libm::log10f(1.0 + (hz / 700.0))
    }

    /// Converts [mel](https://en.wikipedia.org/wiki/Mel_scale) to hertz.
    pub fn mel_to_hertz(mel: f32) -> f32 {
        assert!(mel >= 0.0);
        700.0 * (libm::powf(10.0, mel / 2595.0) - 1.0)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_calculate_y_coord_between_points() {
            assert_eq!(
                // expected y coordinate
                0.5,
                calculate_y_coord_between_points(
                    (100.0, 1.0),
                    (200.0, 0.0),
                    150.0,
                ),
                "Must calculate middle point between points by laying a linear function through the two points"
            );
            // Must calculate arbitrary point between points by laying a linear function through the
            // two points.
            float_cmp::assert_approx_eq!(
                f32,
                0.2,
                calculate_y_coord_between_points((100.0, 1.0), (200.0, 0.0), 180.0,),
                ulps = 3
            );
        }

        #[test]
        fn test_mel() {
            float_cmp::assert_approx_eq!(f32, hertz_to_mel(0.0), 0.0, epsilon = 0.1);
            float_cmp::assert_approx_eq!(f32, hertz_to_mel(500.0), 607.4, epsilon = 0.1);
            float_cmp::assert_approx_eq!(f32, hertz_to_mel(5000.0), 2363.5, epsilon = 0.1);

            let conv = |hz: f32| mel_to_hertz(hertz_to_mel(hz));

            float_cmp::assert_approx_eq!(f32, conv(0.0), 0.0, epsilon = 0.1);
            float_cmp::assert_approx_eq!(f32, conv(1000.0), 1000.0, epsilon = 0.1);
            float_cmp::assert_approx_eq!(f32, conv(10000.0), 10000.0, epsilon = 0.1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test if a frequency spectrum can be sent to other threads.
    #[test]
    const fn test_impl_send() {
        #[allow(unused)]
        // test if this compiles
        fn consume(s: FrequencySpectrum) {
            let _: &dyn Send = &s;
        }
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_spectrum_basic() {
        let spectrum = vec![
            (0.0_f32, 5.0_f32),
            (50.0, 50.0),
            (100.0, 100.0),
            (150.0, 150.0),
            (200.0, 100.0),
            (250.0, 20.0),
            (300.0, 0.0),
            (450.0, 200.0),
            (500.0, 100.0),
        ];

        let mut spectrum_vector = spectrum
            .into_iter()
            .map(|(fr, val)| (fr.into(), val.into()))
            .collect::<Vec<(Frequency, FrequencyValue)>>();

        let spectrum = FrequencySpectrum::new(
            spectrum_vector.clone(),
            50.0,
            spectrum_vector.len() as _,
            &mut spectrum_vector,
        );

        // test inner vector is ordered
        {
            assert_eq!(
                (0.0.into(), 5.0.into()),
                spectrum.data()[0],
                "Vector must be ordered"
            );
            assert_eq!(
                (50.0.into(), 50.0.into()),
                spectrum.data()[1],
                "Vector must be ordered"
            );
            assert_eq!(
                (100.0.into(), 100.0.into()),
                spectrum.data()[2],
                "Vector must be ordered"
            );
            assert_eq!(
                (150.0.into(), 150.0.into()),
                spectrum.data()[3],
                "Vector must be ordered"
            );
            assert_eq!(
                (200.0.into(), 100.0.into()),
                spectrum.data()[4],
                "Vector must be ordered"
            );
            assert_eq!(
                (250.0.into(), 20.0.into()),
                spectrum.data()[5],
                "Vector must be ordered"
            );
            assert_eq!(
                (300.0.into(), 0.0.into()),
                spectrum.data()[6],
                "Vector must be ordered"
            );
            assert_eq!(
                (450.0.into(), 200.0.into()),
                spectrum.data()[7],
                "Vector must be ordered"
            );
            assert_eq!(
                (500.0.into(), 100.0.into()),
                spectrum.data()[8],
                "Vector must be ordered"
            );
        }

        // test DC component getter
        assert_eq!(
            Some(5.0.into()),
            spectrum.dc_component(),
            "Spectrum must contain DC component"
        );

        // test getters
        {
            assert_eq!(0.0, spectrum.min_fr().val(), "min_fr() must work");
            assert_eq!(500.0, spectrum.max_fr().val(), "max_fr() must work");
            assert_eq!(
                (300.0.into(), 0.0.into()),
                spectrum.min(),
                "min() must work"
            );
            assert_eq!(
                (450.0.into(), 200.0.into()),
                spectrum.max(),
                "max() must work"
            );
            assert_eq!(200.0 - 0.0, spectrum.range().val(), "range() must work");
            assert_eq!(80.55556, spectrum.average().val(), "average() must work");
            assert_eq!(
                (50 + 100) as f32 / 2.0,
                spectrum.median().val(),
                "median() must work"
            );
            assert_eq!(
                50.0,
                spectrum.frequency_resolution(),
                "frequency resolution must be returned"
            );
        }

        // test get frequency exact
        {
            assert_eq!(5.0, spectrum.freq_val_exact(0.0).val(),);
            assert_eq!(50.0, spectrum.freq_val_exact(50.0).val(),);
            assert_eq!(150.0, spectrum.freq_val_exact(150.0).val(),);
            assert_eq!(100.0, spectrum.freq_val_exact(200.0).val(),);
            assert_eq!(20.0, spectrum.freq_val_exact(250.0).val(),);
            assert_eq!(0.0, spectrum.freq_val_exact(300.0).val(),);
            assert_eq!(100.0, spectrum.freq_val_exact(375.0).val(),);
            assert_eq!(200.0, spectrum.freq_val_exact(450.0).val(),);
        }

        // test get frequency closest
        {
            assert_eq!((0.0.into(), 5.0.into()), spectrum.freq_val_closest(0.0),);
            assert_eq!((50.0.into(), 50.0.into()), spectrum.freq_val_closest(50.0),);
            assert_eq!(
                (450.0.into(), 200.0.into()),
                spectrum.freq_val_closest(450.0),
            );
            assert_eq!(
                (450.0.into(), 200.0.into()),
                spectrum.freq_val_closest(448.0),
            );
            assert_eq!(
                (450.0.into(), 200.0.into()),
                spectrum.freq_val_closest(400.0),
            );
            assert_eq!((50.0.into(), 50.0.into()), spectrum.freq_val_closest(47.3),);
            assert_eq!((50.0.into(), 50.0.into()), spectrum.freq_val_closest(51.3),);
        }
    }

    #[test]
    #[should_panic]
    fn test_spectrum_get_frequency_value_exact_panic_below_min() {
        let mut spectrum_vector = vec![
            (0.0_f32.into(), 5.0_f32.into()),
            (450.0.into(), 200.0.into()),
        ];

        let spectrum = FrequencySpectrum::new(
            spectrum_vector.clone(),
            50.0,
            spectrum_vector.len() as _,
            &mut spectrum_vector,
        );

        // -1 not included, expect panic
        spectrum.freq_val_exact(-1.0).val();
    }

    #[test]
    #[should_panic]
    fn test_spectrum_get_frequency_value_exact_panic_below_max() {
        let mut spectrum_vector = vec![
            (0.0_f32.into(), 5.0_f32.into()),
            (450.0.into(), 200.0.into()),
        ];

        let spectrum = FrequencySpectrum::new(
            spectrum_vector.clone(),
            50.0,
            spectrum_vector.len() as _,
            &mut spectrum_vector,
        );

        // 451 not included, expect panic
        spectrum.freq_val_exact(451.0).val();
    }

    #[test]
    #[should_panic]
    fn test_spectrum_get_frequency_value_closest_panic_below_min() {
        let mut spectrum_vector = vec![
            (0.0_f32.into(), 5.0_f32.into()),
            (450.0.into(), 200.0.into()),
        ];

        let spectrum = FrequencySpectrum::new(
            spectrum_vector.clone(),
            50.0,
            spectrum_vector.len() as _,
            &mut spectrum_vector,
        );
        // -1 not included, expect panic
        let _ = spectrum.freq_val_closest(-1.0);
    }

    #[test]
    #[should_panic]
    fn test_spectrum_get_frequency_value_closest_panic_below_max() {
        let mut spectrum_vector = vec![
            (0.0_f32.into(), 5.0_f32.into()),
            (450.0.into(), 200.0.into()),
        ];

        let spectrum = FrequencySpectrum::new(
            spectrum_vector.clone(),
            50.0,
            spectrum_vector.len() as _,
            &mut spectrum_vector,
        );

        // 451 not included, expect panic
        let _ = spectrum.freq_val_closest(451.0);
    }

    #[test]
    fn test_nan_safety() {
        let mut spectrum_vector: Vec<(Frequency, FrequencyValue)> =
            vec![(0.0.into(), 0.0.into()); 8];

        let spectrum = FrequencySpectrum::new(
            spectrum_vector.clone(),
            // not important here, any value
            50.0,
            spectrum_vector.len() as _,
            &mut spectrum_vector,
        );

        assert_ne!(
            f32::NAN,
            spectrum.min().1.val(),
            "NaN is not valid, must be 0.0!"
        );
        assert_ne!(
            f32::NAN,
            spectrum.max().1.val(),
            "NaN is not valid, must be 0.0!"
        );
        assert_ne!(
            f32::NAN,
            spectrum.average().val(),
            "NaN is not valid, must be 0.0!"
        );
        assert_ne!(
            f32::NAN,
            spectrum.median().val(),
            "NaN is not valid, must be 0.0!"
        );

        assert_ne!(
            f32::INFINITY,
            spectrum.min().1.val(),
            "INFINITY is not valid, must be 0.0!"
        );
        assert_ne!(
            f32::INFINITY,
            spectrum.max().1.val(),
            "INFINITY is not valid, must be 0.0!"
        );
        assert_ne!(
            f32::INFINITY,
            spectrum.average().val(),
            "INFINITY is not valid, must be 0.0!"
        );
        assert_ne!(
            f32::INFINITY,
            spectrum.median().val(),
            "INFINITY is not valid, must be 0.0!"
        );
    }

    #[test]
    fn test_no_dc_component() {
        let mut spectrum_vector: Vec<(Frequency, FrequencyValue)> =
            vec![(150.0.into(), 150.0.into()), (200.0.into(), 100.0.into())];

        let spectrum = FrequencySpectrum::new(
            spectrum_vector.clone(),
            50.0,
            spectrum_vector.len() as _,
            &mut spectrum_vector,
        );

        assert!(
            spectrum.dc_component().is_none(),
            "This spectrum should not contain a DC component!"
        )
    }

    #[test]
    fn test_max() {
        let maximum: (Frequency, FrequencyValue) = (34.991455.into(), 86.791145.into());
        let mut spectrum_vector: Vec<(Frequency, FrequencyValue)> = vec![
            (2.6916504.into(), 22.81816.into()),
            (5.383301.into(), 2.1004658.into()),
            (8.074951.into(), 8.704016.into()),
            (10.766602.into(), 3.4043686.into()),
            (13.458252.into(), 8.649045.into()),
            (16.149902.into(), 9.210494.into()),
            (18.841553.into(), 14.937911.into()),
            (21.533203.into(), 5.1524887.into()),
            (24.224854.into(), 20.706167.into()),
            (26.916504.into(), 8.359295.into()),
            (29.608154.into(), 3.7514696.into()),
            (32.299805.into(), 15.109907.into()),
            maximum,
            (37.683105.into(), 52.140736.into()),
            (40.374756.into(), 24.108875.into()),
            (43.066406.into(), 11.070151.into()),
            (45.758057.into(), 10.569871.into()),
            (48.449707.into(), 6.1969466.into()),
            (51.141357.into(), 16.722788.into()),
            (53.833008.into(), 8.93011.into()),
        ];

        let spectrum = FrequencySpectrum::new(
            spectrum_vector.clone(),
            44100.0,
            spectrum_vector.len() as _,
            &mut spectrum_vector,
        );

        assert_eq!(
            spectrum.max(),
            maximum,
            "Should return the maximum frequency value!"
        )
    }

    #[test]
    fn test_mel_getter() {
        let mut spectrum_vector = vec![
            (0.0_f32.into(), 5.0_f32.into()),
            (450.0.into(), 200.0.into()),
        ];

        let spectrum = FrequencySpectrum::new(
            spectrum_vector.clone(),
            50.0,
            spectrum_vector.len() as _,
            &mut spectrum_vector,
        );
        let _ = spectrum.mel_val(450.0);
    }
}
