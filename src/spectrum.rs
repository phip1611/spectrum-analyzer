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
//! Module for the struct [`FrequencySpectrum`].

use crate::error::SpectrumAnalyzerError;
use crate::frequency::{Frequency, FrequencyValue};
use crate::scaling::{SpectrumDataStats, SpectrumScalingFunction};
use crate::util::AverageBucket;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cell::{Cell, Ref, RefCell};

/// Convenient wrapper around the processed FFT result which describes each frequency and
/// its value/amplitude in the analyzed slice of samples. It only consists of the frequencies
/// which were desired, e.g. specified via
/// [`crate::limit::FrequencyLimit`] when [`crate::samples_fft_to_spectrum`] was called.
///
/// This means, the spectrum can cover all data from the DC component (0Hz) to the
/// Nyquist frequency.
///
/// All results are related to the sampling rate provided to the library function which
/// creates objects of this struct!
#[derive(Debug, Default)]
pub struct FrequencySpectrum {
    /// Raw data. Vector is sorted from lowest
    /// frequency to highest and data is normalized/scaled
    /// according to all applied scaling functions.
    data: RefCell<Vec<(Frequency, FrequencyValue)>>,
    /// Frequency resolution of the examined samples in Hertz,
    /// i.e the frequency steps between elements in the vector
    /// inside field [`Self::data`].
    frequency_resolution: f32,
    /// Number of samples. This property must be kept separately, because
    /// `data.borrow().len()` might contain less than N elements, if the
    /// spectrum was created with a [`crate::limit::FrequencyLimit`] .
    samples_len: u32,
    /// Average value of frequency value/magnitude/amplitude
    /// corresponding to data in [`FrequencySpectrum::data`].
    average: Cell<FrequencyValue>,
    /// Median value of frequency value/magnitude/amplitude
    /// corresponding to data in [`FrequencySpectrum::data`].
    median: Cell<FrequencyValue>,
    /// Pair of (frequency, frequency value/magnitude/amplitude) where
    /// frequency value is **minimal** inside the spectrum.
    /// Corresponding to data in [`FrequencySpectrum::data`].
    min: Cell<(Frequency, FrequencyValue)>,
    /// Pair of (frequency, frequency value/magnitude/amplitude) where
    /// frequency value is **maximum** inside the spectrum.
    /// Corresponding to data in [`FrequencySpectrum::data`].
    max: Cell<(Frequency, FrequencyValue)>,
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
    #[inline(always)]
    pub fn new(
        data: Vec<(Frequency, FrequencyValue)>,
        frequency_resolution: f32,
        samples_len: u32,
    ) -> Self {
        debug_assert!(
            data.len() >= 2,
            "Input data of length={} for spectrum makes no sense!",
            data.len()
        );

        let obj = Self {
            data: RefCell::new(data),
            frequency_resolution,
            samples_len,
            // default/placeholder values
            average: Cell::new(FrequencyValue::from(-1.0)),
            median: Cell::new(FrequencyValue::from(-1.0)),
            min: Cell::new((Frequency::from(-1.0), FrequencyValue::from(-1.0))),
            max: Cell::new((Frequency::from(-1.0), FrequencyValue::from(-1.0))),
        };
        // IMPORTANT!!
        obj.calc_statistics();
        obj
    }

    /// Applies the function `scaling_fn` to each element and updates
    /// `min`, `max`, etc. afterwards accordingly. It ensures that no value
    /// is `NaN` or `Infinity` afterwards (regarding IEEE-754).
    ///
    /// ## Parameters
    /// * `scaling_fn` See [`crate::scaling::SpectrumScalingFunction`].
    #[inline(always)]
    pub fn apply_scaling_fn(
        &self,
        scaling_fn: SpectrumScalingFunction,
    ) -> Result<(), SpectrumAnalyzerError> {
        {
            // drop RefMut<> from borrow_mut() before calc_statistics
            let mut data = self.data.borrow_mut();

            let stats = SpectrumDataStats {
                min: self.min.get().1.val(),
                max: self.max.get().1.val(),
                average: self.average.get().val(),
                median: self.median.get().val(),
                // attention! not necessarily `data.len()`!
                n: self.samples_len as f32,
            };

            for (_fr, fr_val) in &mut *data {
                // regular for instead of for_each(), so that I can early return a result here
                let scaled_val: f32 = scaling_fn(fr_val.val(), &stats);
                if scaled_val.is_nan() || scaled_val.is_infinite() {
                    return Err(SpectrumAnalyzerError::ScalingError(
                        fr_val.val(),
                        scaled_val,
                    ));
                }
                *fr_val = scaled_val.into()
            }
            // drop RefMut<> from borrow_mut() before calc_statistics
        }

        self.calc_statistics();
        Ok(())
    }

    /// Returns the average frequency value of the spectrum.
    #[inline(always)]
    pub fn average(&self) -> FrequencyValue {
        self.average.get()
    }

    /// Returns the median frequency value of the spectrum.
    #[inline(always)]
    pub fn median(&self) -> FrequencyValue {
        self.median.get()
    }

    /// Returns the maximum (frequency, frequency value)-pair of the spectrum
    /// (regarding the frequency value).
    #[inline(always)]
    pub fn max(&self) -> (Frequency, FrequencyValue) {
        self.max.get()
    }

    /// Returns the minimum (frequency, frequency value)-pair of the spectrum
    /// (regarding the frequency value).
    #[inline(always)]
    pub fn min(&self) -> (Frequency, FrequencyValue) {
        self.min.get()
    }

    /// Returns [`FrequencySpectrum::max().1`] - [`FrequencySpectrum::min().1`],
    /// i.e. the range of the frequency values (not the frequencies itself,
    /// but their amplitudes/values).
    #[inline(always)]
    pub fn range(&self) -> FrequencyValue {
        self.max().1 - self.min().1
    }

    /// Returns the underlying data.
    #[inline(always)]
    pub fn data(&self) -> Ref<Vec<(Frequency, FrequencyValue)>> {
        self.data.borrow()
    }

    /// Returns the frequency resolution of this spectrum.
    #[inline(always)]
    pub const fn frequency_resolution(&self) -> f32 {
        self.frequency_resolution
    }

    /// Returns the number of samples used to obtain this spectrum.
    #[inline(always)]
    pub const fn samples_len(&self) -> u32 {
        self.samples_len
    }

    /// Getter for the highest frequency that is captured inside this spectrum.
    /// Shortcut for `spectrum.data()[spectrum.data().len() - 1].0`.
    /// This corresponds to the [`crate::limit::FrequencyLimit`] of the spectrum.
    ///
    /// This method could return the Nyquist frequency, if there was no Frequency
    /// limit while obtaining the spectrum.
    #[inline(always)]
    pub fn max_fr(&self) -> Frequency {
        let data = self.data.borrow();
        data[data.len() - 1].0
    }

    /// Getter for the lowest frequency that is captured inside this spectrum.
    /// Shortcut for `spectrum.data()[0].0`.
    /// This corresponds to the [`crate::limit::FrequencyLimit`] of the spectrum.
    ///
    /// This method could return the DC component, see [`Self::dc_component`].
    #[inline(always)]
    pub fn min_fr(&self) -> Frequency {
        let data = self.data.borrow();
        data[0].0
    }

    /// Returns the *DC Component* or also called *DC bias* which corresponds
    /// to the FFT result at index 0 which corresponds to `0Hz`. This is only
    /// present if the frequencies were not limited to for example `100 <= f <= 10000`
    /// when the libraries main function was called.
    ///
    /// More information:
    /// https://dsp.stackexchange.com/questions/12972/discrete-fourier-transform-what-is-the-dc-term-really
    ///
    /// Excerpt:
    /// *As far as practical applications go, the DC or 0 Hz term is not particularly useful.
    /// In many cases it will be close to zero, as most signal processing applications will
    /// tend to filter out any DC component at the analogue level. In cases where you might
    /// be interested it can be calculated directly as an average in the usual way, without
    /// resorting to a DFT/FFT.* - Paul R.
    #[inline(always)]
    pub fn dc_component(&self) -> Option<FrequencyValue> {
        let data = self.data.borrow();
        let (maybe_dc_component, dc_value) = &data[0];
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
    #[inline(always)]
    pub fn freq_val_exact(&self, search_fr: f32) -> FrequencyValue {
        let data = self.data.borrow();

        // lowest frequency in the spectrum
        // TODO use minFrequency() and maxFrequency()
        let (min_fr, min_fr_val) = data[0];
        // highest frequency in the spectrum
        let (max_fr, max_fr_val) = data[data.len() - 1];

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

        for two_points in data.iter().as_slice().windows(2) {
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
    #[inline(always)]
    pub fn freq_val_closest(&self, search_fr: f32) -> (Frequency, FrequencyValue) {
        let data = self.data.borrow();

        // lowest frequency in the spectrum
        // TODO use minFrequency() and maxFrequency()
        let (min_fr, min_fr_val) = data[0];
        // highest frequency in the spectrum
        let (max_fr, max_fr_val) = data[data.len() - 1];

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

        for two_points in data.iter().as_slice().windows(2) {
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

    /// Returns a `BTreeMap`. The key is of type u32.
    /// (`f32` is not `Ord`, hence we can't use it as key.) You can optionally specify a
    /// scale function, e.g. multiply all frequencies with 1000 for better
    /// accuracy when represented as unsigned integer.
    ///
    /// ## Parameters
    /// * `scale_fn` optional scale function, e.g. multiply all frequencies with 1000 for better
    ///              accuracy when represented as unsigned integer.
    ///
    /// ## Return
    /// New `BTreeMap` from frequency to frequency value.
    #[inline(always)]
    pub fn to_map(&self, scale_fn: Option<&dyn Fn(f32) -> u32>) -> BTreeMap<u32, f32> {
        self.data
            .borrow()
            .iter()
            .map(|(fr, fr_val)| (fr.val(), fr_val.val()))
            .map(|(fr, fr_val)| {
                (
                    if let Some(fnc) = scale_fn {
                        (fnc)(fr)
                    } else {
                        fr as u32
                    },
                    fr_val,
                )
            })
            .collect()
    }

    /// This is an attempt to return a more "usable" spectrum. If you analyze music,
    /// usually with 44100kHz, you get frequencies 0 to 22050 Hz. Although, the most
    /// interesting and relevant might be frequencies 0 to 2000 Hz. Therefore, this
    /// method returns a sorted vector with all frequencies, but:
    /// - fr 1-128: keep
    /// - fr 129-256: cut number of samples in half (by calculating average of blocks)
    /// - fr 257-512: cut number of samples in half (by calculating average of blocks)
    /// - fr 513-1024: cut number of samples in half (by calculating average of blocks)
    /// - ...
    /// - fr 16385-22050: cut number of samples in half (by calculating average of blocks)
    #[inline(always)]
    pub fn to_log_spectrum(&self) -> Vec<(Frequency, FrequencyValue)> {
        let data = self.data.borrow();
        let mut log_spectrum = Vec::new();

        const FREQUENCY_LOWER_BOUND: f32 = 128.0;
        // initial shrink factor is two, because
        // frequencies 128 < f <= 256 is already shrinked by 2
        let mut shrink_factor = 2;
        let mut frequency_upper_bound = 256.0;

        // condition for the initial frequency, that we want to keep as it is (all data points)
        let initial_fr_cond: fn(&(Frequency, FrequencyValue)) -> bool =
            |(fr, _)| fr.val() <= FREQUENCY_LOWER_BOUND;

        // wee keep all the values for frequencies <=128 Hz
        data.iter()
            .filter(|x| initial_fr_cond(x))
            .for_each(|x| log_spectrum.push(*x));

        // counter for shrinking
        let mut i = 0;
        let mut fr_avg_bucket = AverageBucket::new();
        let mut fr_val_avg_bucket = AverageBucket::new();

        // We iterate all existing data points and accumulate them
        // into the new logarithmic data set (by shrinking the data points
        // on a logarithmic base) (TODO I'm not 100% if this is correct)
        for (fr, fr_val) in data.iter().filter(|x| !initial_fr_cond(x)) {
            // last iteration?
            let is_last = fr == &self.max_fr();

            // this is a little bit ugly but I tried several settings and
            // this works the best I think. A shrink factor of more than 16
            // leads to way too few data points for the high frequencies
            // TODO this whole section needs more work I guess..
            //  it's not sexy and doesn't look like what other
            //  visualization implementations look like :(
            if fr.val() > 128.0 && fr.val() <= 256.0 {
                shrink_factor = 2;
            } else if fr.val() > 256.0 && fr.val() <= 1024.0 {
                shrink_factor = 4;
            } else if fr.val() > 1024.0 {
                shrink_factor = 8;
            }

            // we switch to the next power of 2 shrink block;
            if fr.val() > frequency_upper_bound {
                // in this case, there were not enough data points to reach the shrink factor
                if i != 0 {
                    // add to vector
                    log_spectrum.push((fr_avg_bucket.avg().into(), fr_val_avg_bucket.avg().into()));
                    i = 0;
                }

                // FREQUENCY_LOWER_BOUND = frequency_upper_bound;
                frequency_upper_bound *= 2.0;
                fr_avg_bucket.reset();
                fr_val_avg_bucket.reset();
            }

            fr_avg_bucket.add(fr.val());
            fr_val_avg_bucket.add(fr_val.val());
            i += 1;

            // combine X data points to a single new one
            if i >= shrink_factor {
                i = 0;
                // add to vector
                log_spectrum.push((fr_avg_bucket.avg().into(), fr_val_avg_bucket.avg().into()));
                fr_avg_bucket.reset();
                fr_val_avg_bucket.reset();
            }

            // required because otherwise it might happen that we
            // lose the last element
            if is_last && i != 0 {
                log_spectrum.push((fr_avg_bucket.avg().into(), fr_val_avg_bucket.avg().into()));
            }
        }

        log_spectrum
    }

    /*/// Returns an iterator over the underlying vector [`data`].
    #[inline(always)]
    pub fn iter(&self) -> Iter<(Frequency, FrequencyValue)> {
        self.data.borrow().iter()
    }*/

    /// Calculates min, max, median and average of the frequency values/magnitudes/amplitudes.
    #[inline(always)]
    fn calc_statistics(&self) {
        let mut data_sorted = self.data.borrow().clone();
        data_sorted.sort_by(|(_l_fr, l_fr_val), (_r_fr, r_fr_val)| {
            // compare by frequency value, from min to max
            l_fr_val.cmp(r_fr_val)
        });

        // sum
        let sum: f32 = data_sorted
            .iter()
            .map(|fr_val| fr_val.1.val())
            .fold(0.0, |a, b| a + b);

        let avg = sum / data_sorted.len() as f32;
        let average: FrequencyValue = avg.into();

        let median = {
            // we assume that data_sorted.length() is always even, because
            // it must be a power of 2 (for FFT)
            let a = data_sorted[data_sorted.len() / 2 - 1].1;
            let b = data_sorted[data_sorted.len() / 2].1;
            (a + b) / 2.0.into()
        };

        // because we sorted the vector a few lines above
        // by the value, the following lines are correct
        // i.e. we get min/max value with corresponding frequency
        let min = data_sorted[0];
        let max = data_sorted[data_sorted.len() - 1];

        // check that I get the comparison right (and not from max to min)
        debug_assert!(min.1 <= max.1, "min must be <= max");

        self.min.replace(min);
        self.max.replace(max);
        self.average.replace(average);
        self.median.replace(median);
    }
}

/*impl FromIterator<(Frequency, FrequencyValue)> for FrequencySpectrum {

    #[inline(always)]
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
#[inline(always)]
fn calculate_y_coord_between_points(
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
        assert!(
            // https://docs.rs/float-cmp/0.8.0/float_cmp/
            float_cmp::approx_eq!(
                f32,
                0.2,
                calculate_y_coord_between_points(
                    (100.0, 1.0),
                    (200.0, 0.0),
                    180.0,
                ),
                ulps = 3
            ),
            "Must calculate arbitrary point between points by laying a linear function through the two points"
        );
    }

    #[test]
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
        ];

        let spectrum = spectrum
            .into_iter()
            .map(|(fr, val)| (fr.into(), val.into()))
            .collect::<Vec<(Frequency, FrequencyValue)>>();
        let spectrum_len = spectrum.len() as u32;
        let spectrum = FrequencySpectrum::new(spectrum, 50.0, spectrum_len);

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
        }

        // test DC component getter
        assert!(
            spectrum.dc_component().is_some(),
            "Spectrum must contain DC component"
        );
        assert_eq!(
            5.0,
            spectrum.dc_component().unwrap().val(),
            "Spectrum must contain DC component"
        );

        // test getters
        {
            assert_eq!(0.0, spectrum.min_fr().val(), "min_fr() must work");
            assert_eq!(450.0, spectrum.max_fr().val(), "max_fr() must work");
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
            assert_eq!(78.125, spectrum.average().val(), "average() must work");
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
        let spectrum_vector = vec![(0.0_f32, 5.0_f32), (450.0, 200.0)];

        let spectrum = spectrum_vector
            .into_iter()
            .map(|(fr, val)| (fr.into(), val.into()))
            .collect::<Vec<(Frequency, FrequencyValue)>>();
        let spectrum_len = spectrum.len() as u32;
        let spectrum = FrequencySpectrum::new(spectrum, 50.0, spectrum_len);

        // -1 not included, expect panic
        spectrum.freq_val_exact(-1.0).val();
    }

    #[test]
    #[should_panic]
    fn test_spectrum_get_frequency_value_exact_panic_below_max() {
        let spectrum_vector = vec![(0.0_f32, 5.0_f32), (450.0, 200.0)];

        let spectrum = spectrum_vector
            .into_iter()
            .map(|(fr, val)| (fr.into(), val.into()))
            .collect::<Vec<(Frequency, FrequencyValue)>>();
        let spectrum_len = spectrum.len() as u32;
        let spectrum = FrequencySpectrum::new(spectrum, 50.0, spectrum_len);

        // 451 not included, expect panic
        spectrum.freq_val_exact(451.0).val();
    }

    #[test]
    #[should_panic]
    fn test_spectrum_get_frequency_value_closest_panic_below_min() {
        let spectrum_vector = vec![(0.0_f32, 5.0_f32), (450.0, 200.0)];

        let spectrum = spectrum_vector
            .into_iter()
            .map(|(fr, val)| (fr.into(), val.into()))
            .collect::<Vec<(Frequency, FrequencyValue)>>();
        let spectrum_len = spectrum.len() as u32;
        let spectrum = FrequencySpectrum::new(spectrum, 50.0, spectrum_len);

        // -1 not included, expect panic
        spectrum.freq_val_closest(-1.0);
    }

    #[test]
    #[should_panic]
    fn test_spectrum_get_frequency_value_closest_panic_below_max() {
        let spectrum_vector = vec![(0.0_f32, 5.0_f32), (450.0, 200.0)];

        let spectrum = spectrum_vector
            .into_iter()
            .map(|(fr, val)| (fr.into(), val.into()))
            .collect::<Vec<(Frequency, FrequencyValue)>>();
        let spectrum_len = spectrum.len() as u32;
        let spectrum = FrequencySpectrum::new(spectrum, 50.0, spectrum_len);

        // 451 not included, expect panic
        spectrum.freq_val_closest(451.0);
    }

    #[test]
    fn test_nan_safety() {
        let spectrum_vector: Vec<(Frequency, FrequencyValue)> = vec![(0.0.into(), 0.0.into()); 8];

        let spectrum_len = spectrum_vector.len() as u32;
        let spectrum = FrequencySpectrum::new(
            spectrum_vector,
            // not important here, any valu
            50.0,
            spectrum_len,
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
        let spectrum: Vec<(Frequency, FrequencyValue)> =
            vec![(150.0.into(), 150.0.into()), (200.0.into(), 100.0.into())];

        let spectrum_len = spectrum.len() as u32;
        let spectrum = FrequencySpectrum::new(spectrum, 50.0, spectrum_len);

        assert!(
            spectrum.dc_component().is_none(),
            "This spectrum should not contain a DC component!"
        )
    }

    #[test]
    fn test_to_log_spectrum() {
        let spectrum: Vec<(Frequency, FrequencyValue)> = vec![
            (2.0.into(), 10.0.into()),
            (4.0.into(), 10.0.into()),
            (8.0.into(), 10.0.into()),
            (16.0.into(), 10.0.into()),
            (32.0.into(), 10.0.into()),
            (64.0.into(), 10.0.into()),
            (128.0.into(), 10.0.into()),
            // ---
            (129.0.into(), 20.0.into()),
            (256.0.into(), 10.0.into()),
            // ---
            (257.0.into(), 20.0.into()),
            (512.0.into(), 10.0.into()),
            // ---
            (513.0.into(), 20.0.into()),
            (1024.0.into(), 10.0.into()),
            // ---
            (1025.0.into(), 30.0.into()),
            (2048.0.into(), 10.0.into()),
            // ---
            (2049.0.into(), 30.0.into()),
            (4096.0.into(), 10.0.into()),
            // ---
            (4097.0.into(), 30.0.into()),
            (8192.0.into(), 10.0.into()),
            // ---
            (8193.0.into(), 30.0.into()),
            (16384.0.into(), 10.0.into()),
            // ---
            (16385.0.into(), 30.0.into()),
            (22050.0.into(), 10.0.into()),
        ];

        let spectrum_len = spectrum.len() as u32;
        // frequency resolution is irrelevant
        let spectrum = FrequencySpectrum::new(spectrum, 0.0, spectrum_len);
        let log_spectrum = spectrum.to_log_spectrum();
        // println!("{:#?}", log_spectrum);

        assert_eq!(15, log_spectrum.len(), "Must contain exactly 15 values!");

        let expected = [
            (2.0, 10.0),
            (4.0, 10.0),
            (8.0, 10.0),
            (16.0, 10.0),
            (32.0, 10.0),
            (64.0, 10.0),
            (128.0, 10.0),
            (192.5, 15.0),
            (384.5, 15.0),
            (768.5, 15.0),
            (1536.5, 20.0),
            (3072.5, 20.0),
            (6144.5, 20.0),
            (12288.5, 20.0),
            (19217.5, 20.0),
        ];
        let actual = log_spectrum
            .into_iter()
            .map(|(fr, fr_val)| (fr.val(), fr_val.val()))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        assert_eq!(actual.as_ref(), &expected);
    }
}
