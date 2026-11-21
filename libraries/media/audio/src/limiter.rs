use crate::{Audio, Error, Loudness};

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
    /// Includes the initial limiting attempt; at most three (two corrections).
    pub passes: u8,
    /// Maximum reduction attempted, including a refused attempt.
    pub max_gain_reduction_db: f64,
    pub refused: bool,
    /// Reduction actually applied; zero for static output or refusal.
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
        let mut candidate = self.clone();
        let report = candidate.render_limiter(ceiling_dbtp, config)?;
        if report.max_gain_reduction_db > 6.0 {
            return Err(Error::ReductionLimit { required_db: report.max_gain_reduction_db });
        }
        *self = candidate;
        Ok(report)
    }

    fn render_limiter(&mut self, ceiling_dbtp: f64, config: LimiterConfig) -> Result<LimitReport, Error> {
        config.validate()?;
        if !ceiling_dbtp.is_finite() || ceiling_dbtp > 0.0 {
            return Err(Error::InvalidAudio);
        }
        let ceiling = 10.0_f64.powf((ceiling_dbtp - 0.02) / 20.0);
        if ceiling == 0.0 {
            return Err(Error::InvalidAudio);
        }
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
                    for (tap, k) in kernel.iter().enumerate() {
                        let pos = n as i128 + tap as i128 - 11;
                        if pos >= 0 && pos < frames as i128 {
                            value += self.samples[pos as usize * channels + c] * k;
                        }
                    }
                    if !value.is_finite() { return Err(Error::InvalidAudio); }
                    peak = peak.max(value.abs());
                }
            }
            envelope.push(if peak > ceiling { ceiling / peak } else { 1.0 });
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
            minimum = minimum.min(gain);
            for x in frame {
                *x *= gain;
            }
        }
        let measured = self.measure()?;
        let trim_db = (ceiling_dbtp - 0.02 - measured.true_peak_dbtp).min(0.0);
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
    /// Beyond 6 dB, returns exactly the legacy capped-static samples and a refusal.
    pub fn normalise_to_target(
        &mut self,
        target_lufs: f64,
        ceiling_dbtp: f64,
        config: LimiterConfig,
    ) -> Result<TargetReport, Error> {
        config.validate()?;
        let original = self.clone();
        let static_report = self.normalise(target_lufs, ceiling_dbtp)?;
        let mut report = TargetReport {
            before: static_report.before,
            after: static_report.after,
            gain_db: static_report.gain_db,
            passes: 0,
            max_gain_reduction_db: 0.0,
            refused: false,
            applied_gain_reduction_db: 0.0,
            target_reached: (target_lufs - static_report.after.integrated_lufs).abs() <= 0.5,
            shortfall_lu: static_report.shortfall_lu,
        };
        if !static_report.peak_limited {
            return Ok(report);
        }
        let mut gain_db = target_lufs - report.before.integrated_lufs;
        for pass in 1..=3 {
            let gain = 10.0_f64.powf(gain_db / 20.0);
            let mut candidate = original.clone();
            if !gain.is_finite() || candidate.samples.iter().any(|x| !(x * gain).is_finite()) {
                return Err(Error::InvalidAudio);
            }
            for x in &mut candidate.samples {
                *x *= gain;
            }
            let limited = candidate.render_limiter(ceiling_dbtp, config)?;
            report.passes = pass;
            report.max_gain_reduction_db = report
                .max_gain_reduction_db
                .max(limited.max_gain_reduction_db);
            if limited.max_gain_reduction_db > 6.0 {
                report.refused = true;
                return Ok(report);
            }
            let error = target_lufs - limited.after.integrated_lufs;
            if error.abs() <= 0.5 || pass == 3 {
                report.after = limited.after;
                report.applied_gain_reduction_db = limited.max_gain_reduction_db;
                report.target_reached = error.abs() <= 0.5;
                report.gain_db = gain_db;
                report.shortfall_lu = error.max(0.0);
                *self = candidate;
                return Ok(report);
            }
            gain_db += error;
        }
        unreachable!()
    }
}
