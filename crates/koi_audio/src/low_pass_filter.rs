// This file is partially adapted from fundsp here:
// https://github.com/SamiPerttu/fundsp/blob/master/src/filter.rs
// Licensed Apache 2.0 or MIT

use core::cell::Cell;
use oddio::{Frame, Signal};

#[derive(Copy, Clone, Debug, Default)]
pub struct BiquadCoefs {
    pub a1: f32,
    pub a2: f32,
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
}

impl BiquadCoefs {
    fn interpolate(&self, other: &Self, amount: f32) -> Self {
        Self {
            a1: (other.a1 - self.a1) * amount + self.a1,
            a2: (other.a2 - self.a2) * amount + self.a2,
            b0: (other.b0 - self.b0) * amount + self.b0,
            b1: (other.b1 - self.b1) * amount + self.b1,
            b2: (other.b2 - self.b2) * amount + self.b2,
        }
    }
    /// Returns settings for a Butterworth lowpass filter.
    /// Cutoff is the -3 dB point of the filter in Hz.
    pub fn butterworth_lowpass(inverse_sample_rate: f32, cutoff_hz: f32) -> Self {
        use core::f32::consts::{PI, SQRT_2};

        let f = f32::tan(cutoff_hz * PI * inverse_sample_rate);
        let a0r = 1.0 / (1.0 + SQRT_2 * f + f * f);
        let a1 = (2.0 * f * f - 2.0) * a0r;
        let a2 = (1.0 - SQRT_2 * f + f * f) * a0r;
        let b0 = f * f * a0r;
        let b1 = 2.0 * b0;
        let b2 = b0;
        Self { a1, a2, b0, b1, b2 }
    }
}

// 2nd order IIR filter implemented in normalized Direct Form I.
/// Setting: coefficients as tuple (a1, a2, b0, b1, b2).
/// - Input 0: input signal.
/// - Output 0: filtered signal.
#[derive(Default, Clone)]
pub struct Biquad<InnerSignal: oddio::Signal> {
    inner_signal: InnerSignal,
    pub old_coefs: Cell<BiquadCoefs>,
    pub coefs: Cell<BiquadCoefs>,
    inner_values: Cell<InnerValues>,
}

#[derive(Clone, Copy, Debug)]
struct InnerValues {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Default for InnerValues {
    fn default() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
}

impl<T: oddio::Signal> Biquad<T> {
    pub fn new(inner_signal: T, coefficients: BiquadCoefs) -> Self {
        Self {
            inner_signal,
            old_coefs: Cell::new(coefficients),
            coefs: Cell::new(coefficients),
            inner_values: Cell::new(Default::default()),
        }
    }
}

impl<T: oddio::Signal> oddio::Signal for Biquad<T>
where
    T::Frame: oddio::Frame,
{
    type Frame = T::Frame;

    fn sample(&self, interval: f32, out: &mut [Self::Frame]) {
        self.inner_signal.sample(interval, out);

        let old_coefs = self.old_coefs.get();
        let new_coefs = self.coefs.get();

        let frame_len = out.len();
        for (i, frame) in out.iter_mut().enumerate() {
            let BiquadCoefs { a1, a2, b0, b1, b2 } =
                old_coefs.interpolate(&new_coefs, (i / frame_len) as f32);

            let InnerValues { x1, x2, y1, y2 } = self.inner_values.get();

            let x = frame.channels_mut()[0];
            let y = b0 * x + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
            frame.channels_mut()[0] = y;

            self.inner_values.set(InnerValues {
                x2: x1,
                x1: x,
                y2: y1,
                y1: y,
            });
        }
        self.old_coefs.set(new_coefs);
    }
}

pub struct BiquadControl<'a>(&'a Cell<BiquadCoefs>);

impl<'a> BiquadControl<'a> {
    pub fn set_coeffs(&mut self, biquad_coefs: BiquadCoefs) {
        self.0.set(biquad_coefs)
    }
}

unsafe impl<'a, T: 'a + Signal> oddio::Controlled<'a> for Biquad<T> {
    type Control = BiquadControl<'a>;

    unsafe fn make_control(signal: &'a Biquad<T>) -> Self::Control {
        BiquadControl(&signal.coefs)
    }
}

// TODO: This implementation is obviously incorrect.
impl<T: oddio::Seek> oddio::Seek for Biquad<T>
where
    T::Frame: oddio::Frame,
{
    fn seek(&self, seconds: f32) {
        self.inner_signal.seek(seconds)
    }
}

pub struct LowpassFilter<InnerSignal: oddio::Signal> {
    biquad: Biquad<InnerSignal>,
    inverse_sample_rate: Cell<f32>,
    cutoff_hz: Cell<f32>,
}

impl<InnerSignal: oddio::Signal> LowpassFilter<InnerSignal> {
    /// A Butterworth lowpass filter. Frequences below cutoff_hz are allowed to pass.
    pub fn new(cutoff_hz: f32, inner_signal: InnerSignal) -> Self {
        let inverse_sample_rate = 1.0 / 44100.0;
        Self {
            biquad: Biquad::new(
                inner_signal,
                BiquadCoefs::butterworth_lowpass(inverse_sample_rate, cutoff_hz),
            ),
            inverse_sample_rate: Cell::new(inverse_sample_rate),
            cutoff_hz: Cell::new(cutoff_hz),
        }
    }
}

impl<T: oddio::Signal> oddio::Signal for LowpassFilter<T>
where
    T::Frame: oddio::Frame,
{
    type Frame = T::Frame;

    fn sample(&self, interval: f32, out: &mut [Self::Frame]) {
        if interval != self.inverse_sample_rate.get() {
            self.inverse_sample_rate.set(interval);
            self.biquad.coefs.set(BiquadCoefs::butterworth_lowpass(
                interval,
                self.cutoff_hz.get(),
            ))
        }
        self.biquad.sample(interval, out)
    }
}

pub struct LowpassFilterControl<'a>(&'a Cell<f32>, BiquadControl<'a>);

impl<'a> LowpassFilterControl<'a> {
    pub fn set_cutoff_hz(&mut self, cutoff_hz: f32) {
        self.1
            .set_coeffs(BiquadCoefs::butterworth_lowpass(self.0.get(), cutoff_hz));
    }
}

unsafe impl<'a, T: 'a + Signal> oddio::Controlled<'a> for LowpassFilter<T> {
    type Control = LowpassFilterControl<'a>;

    unsafe fn make_control(signal: &'a Self) -> Self::Control {
        LowpassFilterControl(
            &signal.inverse_sample_rate,
            oddio::Controlled::<'a>::make_control(&signal.biquad),
        )
    }
}
