//! Unit step spring with a conservative displacement and velocity envelope.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringCurve {
    w: f64,
    a: f64,
    wd: f64,
    velocity: f64,
    duration: f64,
}

impl SpringCurve {
    /// Physical damping coefficient (not damping ratio). Only critical and
    /// underdamped, decaying springs have a finite duration here.
    pub fn new(k: f64, damping: f64, mass: f64, velocity: f64) -> Result<Self, &'static str> {
        if ![k, damping, mass, velocity].iter().all(|v| v.is_finite())
            || k <= 0.0 || mass <= 0.0 || damping <= 0.0 {
            return Err("spring requires finite positive stiffness, damping and mass");
        }
        let w = libm::sqrt(k / mass);
        let a = damping / (2.0 * mass);
        if !w.is_finite() || w <= 0.0 || !a.is_finite() || a <= 0.0 || a > w * (1.0 + 1e-12) {
            return Err("spring must be critical or underdamped");
        }
        let wd = libm::sqrt((w * w - a * a).max(0.0));
        if !wd.is_finite() || !(1.0 / a).is_finite() {
            return Err("spring parameters exceed numerical range");
        }
        let mut result = Self { w, a, wd, velocity, duration: 0.0 };
        // Envelope bounds both |x| and |v| for all subsequent times.
        let envelope = |t: f64| {
            let e = libm::exp(-a * t);
            if wd == 0.0 {
                let c = velocity - w;
                e * (1.0 + c.abs() * t).max(velocity.abs() + w * c.abs() * t)
            } else {
                let b = (velocity - a) / wd;
                e * (1.0 + b.abs()).max(velocity.abs() + (wd - a * b).abs())
            }
        };
        let mut hi = (1.0 / a).max(1.0 / w);
        // For critical damping the linear envelope peaks before 1/a.
        if !envelope(hi).is_finite() { return Err("spring envelope overflow"); }
        while envelope(hi) > 1e-4 {
            hi *= 2.0;
            if !hi.is_finite() { return Err("spring duration overflow"); }
        }
        let mut lo = 1.0 / a;
        for _ in 0..64 {
            let mid = (lo + hi) * 0.5;
            if envelope(mid) > 1e-4 { lo = mid; } else { hi = mid; }
        }
        result.duration = hi;
        Ok(result)
    }

    /// Seconds until both position and velocity envelopes are below 1e-4.
    pub fn settle_duration(&self) -> f64 { self.duration }

    /// Progress from 0 towards 1, with initial velocity in units per second.
    pub fn sample(&self, seconds: f64) -> f64 {
        if seconds <= 0.0 { return 0.0; }
        if seconds >= self.duration { return 1.0; }
        let e = libm::exp(-self.a * seconds);
        let x = if self.wd == 0.0 {
            (-1.0 + (self.velocity - self.w) * seconds) * e
        } else {
            let phase = self.wd * seconds;
            e * (-libm::cos(phase) + (self.velocity - self.a) / self.wd * libm::sin(phase))
        };
        1.0 + x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn springs_settle_and_overshoot() {
        for damping in [6.0, 20.0] {
            let spring = SpringCurve::new(100.0, damping, 1.0, 0.0).unwrap();
            assert!(spring.settle_duration().is_finite());
            assert_eq!(spring.sample(0.0), 0.0);
            assert_eq!(spring.sample(spring.settle_duration()), 1.0);
            assert!((spring.sample(spring.settle_duration() - 1e-6) - 1.0).abs() < 1e-4);
        }
        let spring = SpringCurve::new(100.0, 6.0, 1.0, 0.0).unwrap();
        assert!(spring.sample(0.4) > 1.0);
        assert!(SpringCurve::new(100.0, 0.0, 1.0, 0.0).is_err());
        assert!(SpringCurve::new(1.0, 1.0, 1e308, 0.0).is_err());
    }
}
