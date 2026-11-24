use crate::{Audio, Error, Loudness};

/// Final delivery tolerance; PCM aims tighter to leave room for codec error.
pub const TARGET_TOLERANCE_LU: f64 = 0.1;
pub const MAX_LIMITER_PASSES: u8 = 8;
const PCM_TOLERANCE_LU: f64 = 0.03;

pub(crate) fn cpu_seconds() -> f64 {
    let mut time = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: time points to a valid, writable timespec. Thread CPU excludes
    // unrelated concurrent render work. Processing in this module is serial.
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut time) };
    assert_eq!(rc, 0, "thread CPU clock unavailable");
    time.tv_sec as f64 + time.tv_nsec as f64 * 1e-9
}

pub(crate) fn target_matches(target: f64, measured: f64, tolerance: f64) -> bool {
    // Compare endpoints rather than subtracting: decimal endpoint subtraction
    // can round just outside an inclusive tolerance.
    measured.is_finite() && measured >= target - tolerance && measured <= target + tolerance
}

/// Offline, latency-compensated limiter settings. Both channels share one envelope.
#[derive(Clone, Copy, Debug)]
pub struct LimiterConfig {
    pub lookahead_ms: f64,
    pub release_ms: f64,
}
impl Default for LimiterConfig {
    fn default() -> Self {
        // 100 ms avoids rapid speech-envelope modulation while recovering between words.
        Self {
            lookahead_ms: 5.0,
            release_ms: 100.0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct LimitReport {
    pub max_gain_reduction_db: f64,
    pub after: Loudness,
}
#[derive(Clone, Copy, Debug)]
pub struct TargetReport {
    pub before: Loudness,
    pub after: Loudness,
    pub gain_db: f64,
    /// Includes the initial limiting attempt; at most MAX_LIMITER_PASSES.
    pub passes: u8,
    /// CPU seconds per attempted limiter render; unused entries are zero.
    pub pass_cpu_s: [f64; MAX_LIMITER_PASSES as usize],
    /// Maximum reduction attempted, including a refused attempt.
    pub max_gain_reduction_db: f64,
    /// An attempt exceeded 6 dB; an earlier in-bound attempt may be retained.
    pub refused: bool,
    /// Reduction actually applied; zero when static output is returned.
    pub applied_gain_reduction_db: f64,
    /// Explicit success status, including the bounded correction exhaustion case.
    pub target_reached: bool,
    pub shortfall_lu: f64,
}
impl LimiterConfig {
    fn validate(self) -> Result<(), Error> {
        if !self.lookahead_ms.is_finite()
            || !(0.1..=100.0).contains(&self.lookahead_ms)
            || !self.release_ms.is_finite()
            || !(1.0..=1000.0).contains(&self.release_ms)
        {
            return Err(Error::InvalidAudio);
        }
        Ok(())
    }
}
impl Audio {
    /// Four-phase, 24-tap Hann-windowed sinc detector, including zero-padded edges.
    /// A backwards linear attack anticipates each peak; an exponential release
    /// never exceeds that attack envelope. Samples stay at their original positions.
    /// Remeasuring the rendered signal catches reconstruction peaks introduced by
    /// modulation; a final constant safety trim enforces the meter ceiling.
    pub fn limit_true_peak(
        &mut self,
        ceiling_dbtp: f64,
        config: LimiterConfig,
    ) -> Result<LimitReport, Error> {
        let mut candidate = self.try_clone()?;
        let report = candidate.render_limiter(ceiling_dbtp, config)?;
        if report.max_gain_reduction_db > 6.0 {
            return Err(Error::ReductionLimit {
                required_db: report.max_gain_reduction_db,
            });
        }
        *self = candidate;
        Ok(report)
    }

    fn render_limiter(
        &mut self,
        ceiling_dbtp: f64,
        config: LimiterConfig,
    ) -> Result<LimitReport, Error> {
        config.validate()?;
        if !ceiling_dbtp.is_finite() || ceiling_dbtp > 0.0 {
            return Err(Error::InvalidAudio);
        }
        let ceiling = 10.0_f64.powf((ceiling_dbtp - 0.04) / 20.0);
        if ceiling == 0.0 {
            return Err(Error::InvalidAudio);
        }
        let channels = self.channels as usize;
        let frames = self.frames();
        let mut envelope = self.detector_peaks()?;
        // Hold each intersample requirement over both adjacent source frames.
        for n in (1..frames).rev() {
            envelope[n] = envelope[n].max(envelope[n - 1]);
        }
        for peak in &mut envelope {
            *peak = if *peak > ceiling {
                ceiling / *peak
            } else {
                1.0
            };
        }
        let attack_step = 1.0 / (self.rate as f64 * config.lookahead_ms / 1000.0).max(1.0);
        // Full offline reverse traversal supplies the look-ahead without a delay line.
        for n in (0..frames.saturating_sub(1)).rev() {
            envelope[n] = envelope[n].min(envelope[n + 1] + attack_step);
        }
        let release = (-1.0 / (self.rate as f64 * config.release_ms / 1000.0)).exp();
        let mut gain: f64 = 1.0;
        let mut minimum: f64 = 1.0;
        for (frame, wanted) in self.samples.chunks_exact_mut(channels).zip(envelope) {
            gain = wanted.min(1.0 - (1.0 - gain) * release);
            // Finite release tail: below 0.0001 linear reduction, return exactly
            // to unity (under 0.00087 dB), preserving distant beds bit for bit.
            if wanted == 1.0 && 1.0 - gain < 0.0001 {
                gain = 1.0;
            }
            minimum = minimum.min(gain);
            for x in frame {
                *x *= gain;
            }
        }
        let measured = self.measure()?;
        let reconstructed = self.detector_peaks()?.into_iter().fold(0.0_f64, f64::max);
        let peak_db = measured.true_peak_dbtp.max(20.0 * reconstructed.log10());
        let trim_db = (ceiling_dbtp - 0.02 - peak_db).min(0.0);
        if trim_db < 0.0 {
            let trim = 10.0_f64.powf(trim_db / 20.0);
            for x in &mut self.samples {
                *x *= trim;
            }
        }
        Ok(LimitReport {
            max_gain_reduction_db: -20.0 * minimum.log10() - trim_db,
            after: self.measure()?,
        })
    }

    /// Static gain whenever feasible, otherwise bounded peak limiting, never AGC.
    /// Every attempt starts from the original, avoiding cumulative envelope damage.
    /// A refusal retains the best in-bound attempt, or capped static if none exists.
    /// Errors, including Unmeasurable, preserve input samples exactly.
    pub fn normalise_to_target(
        &mut self,
        target_lufs: f64,
        ceiling_dbtp: f64,
        config: LimiterConfig,
    ) -> Result<TargetReport, Error> {
        let mut working = self.try_clone()?;
        let report = working.normalise_target_work(target_lufs, ceiling_dbtp, config)?;
        *self = working;
        Ok(report)
    }

    fn normalise_target_work(
        &mut self,
        target_lufs: f64,
        ceiling_dbtp: f64,
        config: LimiterConfig,
    ) -> Result<TargetReport, Error> {
        config.validate()?;
        let original = self.try_clone()?;
        let static_report = self.normalise(target_lufs, ceiling_dbtp)?;
        if !static_report.after.integrated_lufs.is_finite()
            || static_report.after.integrated_lufs < -70.0
        {
            return Err(Error::Unmeasurable);
        }
        let mut report = TargetReport {
            before: static_report.before,
            after: static_report.after,
            gain_db: static_report.gain_db,
            passes: 0,
            pass_cpu_s: [0.0; MAX_LIMITER_PASSES as usize],
            max_gain_reduction_db: 0.0,
            refused: false,
            applied_gain_reduction_db: 0.0,
            target_reached: target_matches(
                target_lufs,
                static_report.after.integrated_lufs,
                TARGET_TOLERANCE_LU,
            ),
            shortfall_lu: static_report.shortfall_lu,
        };
        if !static_report.peak_limited {
            return Ok(report);
        }
        let mut gain_db = target_lufs - report.before.integrated_lufs;
        let mut best: Option<(Self, TargetReport)> = None;
        for pass in 1..=MAX_LIMITER_PASSES {
            let start = cpu_seconds();
            let gain = 10.0_f64.powf(gain_db / 20.0);
            let mut candidate = original.try_clone()?;
            if !gain.is_finite() || candidate.samples.iter().any(|x| !(x * gain).is_finite()) {
                return Err(Error::InvalidAudio);
            }
            for x in &mut candidate.samples {
                *x *= gain;
            }
            let limited = candidate.render_limiter(ceiling_dbtp, config)?;
            report.pass_cpu_s[usize::from(pass - 1)] = cpu_seconds() - start;
            if !limited.after.integrated_lufs.is_finite() || limited.after.integrated_lufs < -70.0 {
                return Err(Error::Unmeasurable);
            }
            report.passes = pass;
            report.max_gain_reduction_db = report
                .max_gain_reduction_db
                .max(limited.max_gain_reduction_db);
            if limited.max_gain_reduction_db > 6.0 {
                if let Some((audio, mut accepted)) = best {
                    accepted.refused = true;
                    accepted.target_reached = false;
                    accepted.passes = pass;
                    accepted.pass_cpu_s = report.pass_cpu_s;
                    accepted.max_gain_reduction_db = report.max_gain_reduction_db;
                    *self = audio;
                    return Ok(accepted);
                }
                report.refused = true;
                report.target_reached = false;
                return Ok(report);
            }
            let error = target_lufs - limited.after.integrated_lufs;
            let mut accepted = report;
            accepted.after = limited.after;
            accepted.applied_gain_reduction_db = limited.max_gain_reduction_db;
            accepted.target_reached = target_matches(
                target_lufs,
                limited.after.integrated_lufs,
                TARGET_TOLERANCE_LU,
            );
            accepted.gain_db = gain_db;
            accepted.shortfall_lu = error.max(0.0);
            if best.as_ref().is_none_or(|(_, prior)| {
                error.abs() < (target_lufs - prior.after.integrated_lufs).abs()
            }) {
                best = Some((candidate, accepted));
            }
            if error.abs() <= PCM_TOLERANCE_LU || pass == MAX_LIMITER_PASSES {
                let (audio, mut accepted) = best.expect("an in-bound attempt was recorded");
                accepted.passes = pass;
                accepted.pass_cpu_s = report.pass_cpu_s;
                accepted.max_gain_reduction_db = report.max_gain_reduction_db;
                accepted.refused = !accepted.target_reached;
                *self = audio;
                return Ok(accepted);
            }
            gain_db += error;
        }
        unreachable!()
    }
}

impl Audio {
    fn detector_peaks(&self) -> Result<Vec<f64>, Error> {
        let mut kernels = [[0.0; 24]; 4];
        for (phase, kernel) in kernels.iter_mut().enumerate() {
            for (tap, k) in kernel.iter_mut().enumerate() {
                let x = tap as f64 - 11.0 - phase as f64 / 4.0;
                let sinc = if x == 0.0 {
                    1.0
                } else {
                    (std::f64::consts::PI * x).sin() / (std::f64::consts::PI * x)
                };
                *k = sinc * (0.5 + 0.5 * (std::f64::consts::PI * x / 12.0).cos());
            }
            let sum: f64 = kernel.iter().sum();
            for k in kernel {
                *k /= sum;
            }
        }
        let channels = self.channels as usize;
        let frames = self.frames();
        let mut envelope = Vec::new();
        envelope
            .try_reserve_exact(frames)
            .map_err(|_| Error::SizeOverflow)?;
        for n in 0..frames {
            let mut peak: f64 = 0.0;
            for c in 0..channels {
                peak = peak.max(self.samples[n * channels + c].abs());
                for kernel in &kernels {
                    let mut value = 0.0;
                    let mut reflected = 0.0;
                    for (tap, k) in kernel.iter().enumerate() {
                        let pos = n as i128 + tap as i128 - 11;
                        if pos >= 0 && pos < frames as i128 {
                            value += self.samples[pos as usize * channels + c] * k;
                        }
                        // Resamplers may mirror boundaries rather than zero-pad.
                        // Protect both reconstructions, especially the first burst.
                        let period = 2 * frames.saturating_sub(1) as i128;
                        let r = if period == 0 {
                            0
                        } else {
                            pos.rem_euclid(period)
                        };
                        let r = if r >= frames as i128 { period - r } else { r };
                        reflected += self.samples[r as usize * channels + c] * k;
                    }
                    if !value.is_finite() || !reflected.is_finite() {
                        return Err(Error::InvalidAudio);
                    }
                    peak = peak.max(value.abs());
                    peak = peak.max(reflected.abs());
                }
            }
            envelope.push(peak);
        }
        Ok(envelope)
    }
}

#[cfg(test)]
mod tolerance_tests {
    use super::*;
    #[test]
    fn inclusive_edges_and_nonfinite_refusal() {
        for measured in [-14.1, -13.9, -14.0] {
            assert!(target_matches(-14.0, measured, TARGET_TOLERANCE_LU));
        }
        for measured in [
            -14.100001,
            -13.899999,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            assert!(!target_matches(-14.0, measured, TARGET_TOLERANCE_LU));
        }
    }
}

