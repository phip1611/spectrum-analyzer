//! Module for the struct [`FrequencyLimit`].

/// Can be used to specify a desired frequency limit. If you know that you only
/// need frequencies `f <= 1000Hz`, `1000 <= f <= 6777`, or `10000 <= f`, then this
/// can help you to accelerate overall computation speed and memory usage.
///
/// Please note that due to frequency inaccuracies the FFT result may not contain
/// a value for `1000Hz` but for `998.76Hz`!
#[derive(Debug, Copy, Clone)]
pub enum FrequencyLimit {
    /// Interested in all frequencies. [0, sampling_rate/2] (Nyquist theorem).
    All,
    /// Only interested in frequencies `f <= 1000Hz` for example. Limit is inclusive.
    Min(f32),
    /// Only interested in frequencies `10000 <= f` for example. Limit is inclusive.
    Max(f32),
    /// Only interested in frequencies `1000 <= f <= 6777` for example. Both values are inclusive.
    Range(f32, f32),
}

impl FrequencyLimit {

    #[inline(always)]
    pub fn maybe_min(&self) -> Option<f32> {
        match self {
            FrequencyLimit::Min(min) => Some(*min),
            FrequencyLimit::Range(min, _) => Some(*min),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn maybe_max(&self) -> Option<f32> {
        match self {
            FrequencyLimit::Max(max) => Some(*max),
            FrequencyLimit::Range(_, max) => Some(*max),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn min(&self) -> f32 {
        self.maybe_min().expect("Must contain a value!")
    }

    #[inline(always)]
    pub fn max(&self) -> f32 {
        self.maybe_max().expect("Must contain a value!")
    }
}
