// SPDX-License-Identifier: MIT OR Apache-2.0
//! Toolkit-neutral scalar tracks. Loading allocates; sampling does not.

use crate::{CubicBezier, SpringCurve};
use serde::{Deserialize, Serialize};

pub const PROPERTY_COUNT: usize = 14;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(usize)]
pub enum Property {
    Opacity,
    Tx,
    Ty,
    Scale,
    ScaleX,
    ScaleY,
    Rotate,
    Blur,
    ClipTop,
    ClipRight,
    ClipBottom,
    ClipLeft,
    Scalar,
    DashOffset,
}

impl Property {
    pub const ALL: [Self; PROPERTY_COUNT] = [
        Self::Opacity,
        Self::Tx,
        Self::Ty,
        Self::Scale,
        Self::ScaleX,
        Self::ScaleY,
        Self::Rotate,
        Self::Blur,
        Self::ClipTop,
        Self::ClipRight,
        Self::ClipBottom,
        Self::ClipLeft,
        Self::Scalar,
        Self::DashOffset,
    ];
}

/// Fixed storage and an explicit presence mask. Absent properties read as
/// identity values; callers use `contains` to distinguish them from tracks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Values {
    data: [f64; PROPERTY_COUNT],
    mask: u16,
}

impl Default for Values {
    fn default() -> Self {
        let mut data = [0.0; PROPERTY_COUNT];
        for p in [
            Property::Opacity,
            Property::Scale,
            Property::ScaleX,
            Property::ScaleY,
        ] {
            data[p as usize] = 1.0;
        }
        Self { data, mask: 0 }
    }
}

impl Values {
    pub fn contains(&self, property: Property) -> bool {
        self.mask & (1 << property as usize) != 0
    }
    pub fn get(&self, property: Property) -> f64 {
        self.data[property as usize]
    }
    pub fn set(&mut self, property: Property, value: f64) {
        self.data[property as usize] = value;
        self.mask |= 1 << property as usize;
    }
    pub fn is_empty(&self) -> bool {
        self.mask == 0
    }
    /// Interpolate the union of present properties, using identity for absent
    /// ones. Progress may overshoot; finite extrapolation saturates at ±f64::MAX
    /// when the result cannot be represented. Progress itself is not clamped.
    pub fn lerp(&self, to: &Self, progress: f64) -> Self {
        let mut result = Self::default();
        for property in Property::ALL {
            if self.contains(property) || to.contains(property) {
                result.set(
                    property,
                    lerp(self.get(property), to.get(property), progress),
                );
            }
        }
        result
    }
}

fn lerp(a: f64, b: f64, progress: f64) -> f64 {
    if progress == 0.0 {
        a
    } else if progress == 1.0 {
        b
    }
    // A convex sum avoids overflow in b-a for opposite finite extremes.
    else if (0.0..=1.0).contains(&progress) {
        ((1.0 - progress) * a + progress * b).clamp(a.min(b), a.max(b))
    } else {
        let result = a + (b - a) * progress;
        if result.is_finite() {
            result
        } else {
            // Halving endpoints avoids overflowing their difference and leaves
            // room for cancellation before restoring the result's magnitude.
            let half = a * 0.5 + (b * 0.5 - a * 0.5) * progress;
            (half * 2.0).clamp(-f64::MAX, f64::MAX)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Curve {
    #[default]
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bezier {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    Steps {
        n: u32,
        start: bool,
    },
    Spring {
        stiffness: f64,
        damping: f64,
        mass: f64,
        velocity: f64,
    },
}

impl Curve {
    pub fn validate(&self) -> Result<(), &'static str> {
        match *self {
            Self::Bezier { x1, y1, x2, y2 } => {
                CubicBezier::new(x1, y1, x2, y2).map_err(|_| "invalid cubic bezier")?;
            }
            Self::Steps { n: 0, .. } => return Err("steps must have at least one step"),
            Self::Spring {
                stiffness,
                damping,
                mass,
                velocity,
            } => {
                SpringCurve::new(stiffness, damping, mass, velocity)?;
            }
            _ => {}
        }
        Ok(())
    }
    pub fn sample(&self, progress: f64) -> f64 {
        self.sample_before(progress, false)
    }
    fn sample_before(&self, progress: f64, before: bool) -> f64 {
        let u = progress.clamp(0.0, 1.0);
        match *self {
            Self::Linear => u,
            Self::Ease => CubicBezier::EASE.sample(u),
            Self::EaseIn => CubicBezier::EASE_IN.sample(u),
            Self::EaseOut => CubicBezier::EASE_OUT.sample(u),
            Self::EaseInOut => CubicBezier::EASE_IN_OUT.sample(u),
            Self::Bezier { x1, y1, x2, y2 } => {
                CubicBezier::new(x1, y1, x2, y2).map_or(u, |c| c.sample(u))
            }
            Self::Steps { n, start } => {
                if n == 0 {
                    return u;
                }
                let scaled = u * n as f64;
                let mut step = scaled.floor() + if start { 1.0 } else { 0.0 };
                if before && u == 0.0 {
                    step -= 1.0;
                }
                (step / n as f64).clamp(0.0, 1.0)
            }
            Self::Spring {
                stiffness,
                damping,
                mass,
                velocity,
            } => {
                if u == 0.0 || u == 1.0 {
                    return u;
                }
                SpringCurve::new(stiffness, damping, mass, velocity)
                    .map_or(u, |s| s.sample(u * s.settle_duration()))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keyframe {
    pub offset: f64,
    pub value: f64,
    /// Applies from this keyframe to the next; omitted easing is CSS `ease`.
    #[serde(default = "default_keyframe_easing")]
    pub easing: Curve,
}

fn default_keyframe_easing() -> Curve {
    Curve::Ease
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Track {
    pub property: Property,
    pub keyframes: Vec<Keyframe>,
}

impl Track {
    pub fn sample_at(&self, timing: &Timing, t: f64) -> Values {
        let mut values = Values::default();
        if let Some((progress, before)) = timing.progress(t) {
            values.set(self.property, self.sample(progress, before));
        }
        values
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.keyframes.len() < 2
            || self.keyframes[0].offset != 0.0
            || self.keyframes.last().unwrap().offset != 1.0
        {
            return Err("track needs endpoints at zero and one");
        }
        let mut previous = -1.0;
        for frame in &self.keyframes {
            if !frame.offset.is_finite()
                || frame.offset <= previous
                || frame.offset > 1.0
                || !frame.value.is_finite()
            {
                return Err("keyframes must be finite with strictly increasing unit offsets");
            }
            frame.easing.validate()?;
            previous = frame.offset;
        }
        Ok(())
    }
    fn sample(&self, progress: f64, before: bool) -> f64 {
        // Endpoints are validated on load. Graceful handling here also makes
        // direct construction safe without a panic on an empty track.
        let Some(first) = self.keyframes.first() else {
            return 0.0;
        };
        if progress < first.offset {
            return first.value;
        }
        let end = self
            .keyframes
            .partition_point(|frame| frame.offset <= progress);
        let index = end
            .saturating_sub(1)
            .min(self.keyframes.len().saturating_sub(2));
        let a = &self.keyframes[index];
        let Some(b) = self.keyframes.get(index + 1) else {
            return a.value;
        };
        let local = (progress - a.offset) / (b.offset - a.offset);
        lerp(a.value, b.value, a.easing.sample_before(local, before))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fill {
    #[default]
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    Normal,
    Reverse,
    Alternate,
    #[serde(rename = "alternate-reverse", alias = "alternate_reverse")]
    AlternateReverse,
}

impl Direction {
    fn reverses(self, iteration: f64) -> bool {
        match self {
            Self::Normal => false,
            Self::Reverse => true,
            Self::Alternate => iteration % 2.0 == 1.0,
            Self::AlternateReverse => iteration % 2.0 == 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "count", rename_all = "snake_case")]
pub enum Iterations {
    Finite(f64),
    Infinite,
}
impl Default for Iterations {
    fn default() -> Self {
        Self::Finite(1.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Timing {
    pub duration: f64,
    pub delay: f64,
    pub stagger: f64,
    pub index: u32,
    /// Absolute start of the chosen beat, resolved by the caller. `at_beat`
    /// uses the deck's last-beat clamping rule.
    pub beat_start: f64,
    pub iterations: Iterations,
    pub fill: Fill,
    pub direction: Direction,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            duration: 1.0,
            delay: 0.0,
            stagger: 0.0,
            index: 0,
            beat_start: 0.0,
            iterations: Iterations::default(),
            fill: Fill::None,
            direction: Direction::Normal,
        }
    }
}

impl Timing {
    pub fn at_beat(mut self, beats: &[f64], beat: usize) -> Self {
        self.beat_start = beats
            .get(beat.min(beats.len().saturating_sub(1)))
            .copied()
            .unwrap_or(0.0);
        self
    }
    pub fn start(&self) -> f64 {
        self.beat_start + self.delay + self.index as f64 * self.stagger
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if ![
            self.duration,
            self.delay,
            self.stagger,
            self.beat_start,
            self.start(),
        ]
        .iter()
        .all(|v| v.is_finite())
            || self.duration < 0.0
        {
            return Err("timing must be finite and duration non-negative");
        }
        if let Iterations::Finite(n) = self.iterations
            && (!n.is_finite() || n < 0.0 || !(n * self.duration).is_finite())
        {
            return Err("iteration count must be finite and non-negative");
        }
        if self.duration == 0.0 && self.iterations == Iterations::Infinite {
            return Err("zero duration cannot repeat infinitely");
        }
        Ok(())
    }
    /// Returns directed progress and the CSS before flag, or no filled value.
    fn progress(&self, t: f64) -> Option<(f64, bool)> {
        if !t.is_finite() {
            return None;
        }
        let start = self.start();
        let elapsed = t - start;
        let before = t < start;
        let total = match self.iterations {
            Iterations::Finite(n) => n,
            Iterations::Infinite => f64::INFINITY,
        };
        let after = !before
            && (self.duration == 0.0
                || matches!(self.iterations, Iterations::Finite(_))
                    && elapsed >= self.duration * total);
        if before && !matches!(self.fill, Fill::Backwards | Fill::Both)
            || after && !matches!(self.fill, Fill::Forwards | Fill::Both)
        {
            return None;
        }
        // Reduce time before dividing. `elapsed / duration` may overflow or
        // lose iteration parity for long-running infinite animations.
        if !before && self.iterations == Iterations::Infinite && self.duration > 0.0 {
            if matches!(self.direction, Direction::Normal | Direction::Reverse) {
                let phase = cycle_phase(t, start, self.duration);
                return Some((
                    if self.direction == Direction::Reverse {
                        1.0 - phase
                    } else {
                        phase
                    },
                    false,
                ));
            }
            let cycle = self.duration * 2.0;
            let directed = if cycle.is_finite() {
                cycle_phase(t, start, cycle) * 2.0
            } else if elapsed.is_finite() {
                elapsed / self.duration
            } else {
                (t / self.duration - start / self.duration) % 2.0
            };
            let phase = if directed > 1.0 {
                2.0 - directed
            } else {
                directed
            };
            return Some((
                if self.direction == Direction::AlternateReverse {
                    1.0 - phase
                } else {
                    phase
                },
                false,
            ));
        }
        let iteration_progress = if before {
            0.0
        } else if after {
            total
        } else {
            elapsed / self.duration
        };
        let mut iteration = iteration_progress.floor();
        let mut progress = iteration_progress - iteration;
        if after && iteration_progress > 0.0 && progress == 0.0 {
            iteration -= 1.0;
            progress = 1.0;
        }
        let reverse = self.direction.reverses(iteration);
        if reverse {
            progress = 1.0 - progress;
        }
        Some((progress, before))
    }
}

/// Unit phase of elapsed time without overflowing either the subtraction or
/// division. Separate remainder reduction is needed only for opposite extremes.
fn cycle_phase(t: f64, start: f64, period: f64) -> f64 {
    let elapsed = t - start;
    if elapsed.is_finite() {
        return (elapsed % period) / period;
    }
    let phase = ((t % period) / period - (start % period) / period) % 1.0;
    if phase < 0.0 { phase + 1.0 } else { phase }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub name: String,
    pub timing: Timing,
    pub tracks: Vec<Track>,
}

impl Preset {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.timing.validate()?;
        let mut seen = 0_u16;
        for track in &self.tracks {
            let bit = 1 << track.property as usize;
            if seen & bit != 0 {
                return Err("duplicate property track");
            }
            seen |= bit;
            track.validate()?;
        }
        Ok(())
    }
    pub fn sample(&self, t: f64) -> Values {
        sample(self, &self.timing, t)
    }
}

/// Pure sampling of validated data. No clock, mutable state or heap allocation.
pub fn sample(preset: &Preset, timing: &Timing, t: f64) -> Values {
    let mut values = Values::default();
    if let Some((progress, before)) = timing.progress(t) {
        for track in &preset.tracks {
            values.set(track.property, track.sample(progress, before));
        }
    }
    values
}

/// Allocate a new linear transition from the sampled values. Its first sample
/// has the same numeric property values as `current` for nonzero duration.
/// Newly introduced target properties are present with identity values at the
/// start. This preserves position; callers needing velocity
/// continuity can use the existing Hermite or spring primitives.
pub fn retarget(current: &Values, target: &Values, timing: Timing) -> Preset {
    let tracks = Property::ALL
        .into_iter()
        .filter(|p| current.contains(*p) || target.contains(*p))
        .map(|property| Track {
            property,
            keyframes: vec![
                Keyframe {
                    offset: 0.0,
                    value: current.get(property),
                    easing: Curve::Linear,
                },
                Keyframe {
                    offset: 1.0,
                    value: target.get(property),
                    easing: Curve::Linear,
                },
            ],
        })
        .collect();
    Preset {
        name: "retarget".into(),
        timing,
        tracks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn line() -> Preset {
        Preset {
            name: "line".into(),
            timing: Timing {
                fill: Fill::Both,
                ..Timing::default()
            },
            tracks: vec![Track {
                property: Property::Tx,
                keyframes: vec![
                    Keyframe {
                        offset: 0.0,
                        value: 0.0,
                        easing: Curve::Linear,
                    },
                    Keyframe {
                        offset: 1.0,
                        value: 100.0,
                        easing: Curve::Linear,
                    },
                ],
            }],
        }
    }
    #[test]
    fn delay_stagger_beats_and_fill() {
        let p = line();
        let mut timing = Timing {
            delay: 0.2,
            stagger: 0.1,
            index: 3,
            fill: Fill::None,
            ..Timing::default()
        }
        .at_beat(&[0.0, 2.0], 99);
        assert_eq!(timing.start(), 2.5);
        assert!(sample(&p, &timing, 2.0).is_empty());
        assert_eq!(sample(&p, &timing, 3.0).get(Property::Tx), 50.0);
        assert!(sample(&p, &timing, 3.5).is_empty());
        timing.fill = Fill::Forwards;
        assert!(sample(&p, &timing, 2.0).is_empty());
        assert_eq!(sample(&p, &timing, 4.0).get(Property::Tx), 100.0);
        timing.fill = Fill::Backwards;
        assert_eq!(sample(&p, &timing, 2.0).get(Property::Tx), 0.0);
        assert!(sample(&p, &timing, 4.0).is_empty());
    }
    #[test]
    fn alternate_fractional_and_infinite_iterations() {
        let mut p = line();
        p.timing.direction = Direction::Alternate;
        p.timing.iterations = Iterations::Finite(2.5);
        assert_eq!(p.sample(1.25).get(Property::Tx), 75.0);
        assert_eq!(p.sample(99.0).get(Property::Tx), 50.0);
        p.timing.iterations = Iterations::Infinite;
        assert_eq!(p.sample(1_000_001.25).get(Property::Tx), 75.0);
        assert_eq!(p.sample(1_000_002.25).get(Property::Tx), 25.0);
        p.timing.iterations = Iterations::Finite(2.0);
        assert_eq!(p.sample(2.0).get(Property::Tx), 0.0);
        p.timing.iterations = Iterations::Infinite;
        p.timing.duration = 1e-300;
        assert!(p.sample(1e300).get(Property::Tx).is_finite());
    }
    #[test]
    fn steps_fractional_reverse_ending_fill_keeps_midpoint() {
        let mut p = line();
        p.timing.iterations = Iterations::Finite(3.5);
        p.timing.direction = Direction::Alternate;
        p.timing.fill = Fill::Forwards;
        p.tracks[0].keyframes[0].easing = Curve::Steps { n: 2, start: false };
        assert_eq!(p.sample(3.5).get(Property::Tx), 50.0);
        assert_eq!(p.sample(99.0).get(Property::Tx), 50.0);
        assert_eq!(
            Curve::Steps { n: 2, start: false }.sample_before(0.5, true),
            0.5
        );
    }
    #[test]
    fn step_start_after_fill_does_not_use_before_flag() {
        for (direction, counts) in [
            (Direction::Reverse, &[1.0, 2.0, 3.0, 4.0][..]),
            (Direction::Alternate, &[2.0, 4.0][..]),
            (Direction::AlternateReverse, &[1.0, 3.0][..]),
        ] {
            for &count in counts {
                for fill in [Fill::Forwards, Fill::Both] {
                    let mut p = line();
                    p.timing.direction = direction;
                    p.timing.iterations = Iterations::Finite(count);
                    p.timing.fill = fill;
                    p.tracks[0].keyframes[0].easing = Curve::Steps { n: 2, start: true };
                    assert_eq!(p.timing.progress(count), Some((0.0, false)));
                    assert_eq!(p.sample(count).get(Property::Tx), 50.0);
                    assert_eq!(p.sample(count + 100.0).get(Property::Tx), 50.0);
                }
            }
        }
    }
    #[test]
    fn step_start_backwards_fill_retains_before_flag() {
        for direction in [Direction::Normal, Direction::Alternate] {
            for fill in [Fill::Backwards, Fill::Both] {
                let mut p = line();
                p.timing.direction = direction;
                p.timing.fill = fill;
                p.timing.delay = 1.0;
                p.tracks[0].keyframes[0].easing = Curve::Steps { n: 2, start: true };
                assert_eq!(p.timing.progress(0.0), Some((0.0, true)));
                assert_eq!(p.sample(0.0).get(Property::Tx), 0.0);
                assert_eq!(p.sample(1.0).get(Property::Tx), 50.0);
            }
        }
    }
    #[test]
    fn omitted_keyframe_easing_is_css_ease() {
        let frame: Keyframe = serde_json::from_str(r#"{"offset":0,"value":0}"#).unwrap();
        assert_eq!(frame.easing, Curve::Ease);
        assert!((frame.easing.sample(0.5) - 0.8024033876).abs() < 1e-8);
        let explicit: Keyframe =
            serde_json::from_str(r#"{"offset":0,"value":0,"easing":{"kind":"linear"}}"#).unwrap();
        assert_eq!(explicit.easing, Curve::Linear);
    }
    #[test]
    fn all_directions_apply_to_active_and_filled_phases() {
        for (direction, before, first, second, after) in [
            (Direction::Normal, 0.0, 25.0, 25.0, 100.0),
            (Direction::Reverse, 100.0, 75.0, 75.0, 0.0),
            (Direction::Alternate, 0.0, 25.0, 75.0, 0.0),
            (Direction::AlternateReverse, 100.0, 75.0, 25.0, 100.0),
        ] {
            let mut p = line();
            p.timing.direction = direction;
            p.timing.iterations = Iterations::Finite(2.0);
            for fill in [Fill::None, Fill::Backwards, Fill::Forwards, Fill::Both] {
                p.timing.fill = fill;
                assert_eq!(p.sample(0.25).get(Property::Tx), first);
                assert_eq!(p.sample(1.25).get(Property::Tx), second);
                if matches!(fill, Fill::Backwards | Fill::Both) {
                    assert_eq!(p.sample(-1.0).get(Property::Tx), before);
                } else {
                    assert!(p.sample(-1.0).is_empty());
                }
                if matches!(fill, Fill::Forwards | Fill::Both) {
                    assert_eq!(p.sample(2.0).get(Property::Tx), after);
                    assert_eq!(p.sample(99.0).get(Property::Tx), after);
                } else {
                    assert!(p.sample(2.0).is_empty());
                }
            }
            p.timing.iterations = Iterations::Infinite;
            assert_eq!(p.sample(1_000_000.25).get(Property::Tx), first);
            assert_eq!(p.sample(1_000_001.25).get(Property::Tx), second);
            let encoded = serde_json::to_string(&direction).unwrap();
            assert_eq!(
                serde_json::from_str::<Direction>(&encoded).unwrap(),
                direction
            );
        }
        assert_eq!(
            serde_json::to_string(&Direction::AlternateReverse).unwrap(),
            r#""alternate-reverse""#
        );
    }
    #[test]
    fn extreme_extrapolation_saturates_without_intermediate_overflow() {
        let mut a = Values::default();
        let mut b = Values::default();
        a.set(Property::Scalar, -1e308);
        b.set(Property::Scalar, 1e308);
        assert_eq!(a.lerp(&b, 2.0).get(Property::Scalar), f64::MAX);
        assert_eq!(a.lerp(&b, -1.0).get(Property::Scalar), -f64::MAX);
        // A representable extrapolation must survive overflowing b-a.
        assert!((a.lerp(&b, 1.1).get(Property::Scalar) / 1.2e308 - 1.0).abs() < 1e-15);
        a.set(Property::Scalar, f64::MAX);
        b.set(Property::Scalar, f64::MAX);
        assert_eq!(a.lerp(&b, f64::MAX).get(Property::Scalar), f64::MAX);
    }
    #[test]
    fn named_curves_and_steps() {
        for (curve, expected) in [
            (Curve::Linear, 0.5),
            (Curve::Ease, 0.8024033876),
            (Curve::EaseIn, 0.3153568126),
            (Curve::EaseOut, 0.6846431874),
            (Curve::EaseInOut, 0.5),
        ] {
            assert!((curve.sample(0.5) - expected).abs() < 1e-8);
        }
        assert_eq!(Curve::Steps { n: 4, start: true }.sample(0.0), 0.25);
        assert_eq!(Curve::Steps { n: 4, start: false }.sample(0.24), 0.0);
        assert_eq!(Curve::Steps { n: 4, start: false }.sample(0.25), 0.25);
        assert_eq!(
            Curve::Steps { n: 4, start: true }.sample_before(0.0, true),
            0.0
        );
        let edge = Curve::Bezier {
            x1: 0.0,
            y1: -2.0,
            x2: 0.0,
            y2: 3.0,
        };
        assert!(edge.sample(0.5).is_finite());
        let edge = Curve::Bezier {
            x1: 1.0,
            y1: -2.0,
            x2: 1.0,
            y2: 3.0,
        };
        assert!(edge.sample(0.5).is_finite());
    }
    #[test]
    fn retarget_and_values_interpolation() {
        let p = line();
        let current = p.sample(0.35);
        let target = p.sample(1.0);
        let transition = retarget(
            &current,
            &target,
            Timing {
                beat_start: 0.35,
                fill: Fill::Both,
                ..Timing::default()
            },
        );
        assert_eq!(transition.sample(0.35), current);
        assert_eq!(transition.sample(1.35), target);
        assert_eq!(current.lerp(&target, 0.0), current);
        assert_eq!(current.lerp(&target, 1.0), target);
    }
    #[test]
    fn finite_extremes_interpolate_without_overflow() {
        let mut a = Values::default();
        a.set(Property::Scalar, -1e308);
        let mut b = Values::default();
        b.set(Property::Scalar, 1e308);
        assert_eq!(a.lerp(&b, 0.5).get(Property::Scalar), 0.0);
        assert_eq!(a.lerp(&b, 0.0), a);
        assert_eq!(a.lerp(&b, 1.0), b);
        a.set(Property::Scalar, f64::MAX);
        b.set(Property::Scalar, f64::MAX);
        assert_eq!(a.lerp(&b, 0.1).get(Property::Scalar), f64::MAX);
        let mut p = line();
        p.tracks[0].keyframes[0].value = -1e308;
        p.tracks[0].keyframes[1].value = 1e308;
        assert_eq!(p.sample(0.5).get(Property::Tx), 0.0);
    }
    #[test]
    fn infinite_phase_handles_elapsed_overflow() {
        let mut p = line();
        p.timing.beat_start = -1e308;
        p.timing.iterations = Iterations::Infinite;
        p.timing.fill = Fill::None;
        for direction in [
            Direction::Normal,
            Direction::Reverse,
            Direction::Alternate,
            Direction::AlternateReverse,
        ] {
            p.timing.direction = direction;
            for duration in [0.125, 1e-300, 1e308] {
                p.timing.duration = duration;
                assert!(p.timing.validate().is_ok());
                let sample = p.sample(1e308);
                assert!(sample.contains(Property::Tx));
                assert!(sample.get(Property::Tx).is_finite());
                assert!((0.0..=100.0).contains(&sample.get(Property::Tx)));
            }
        }
        assert_eq!(cycle_phase(1e308, -1e308, 0.125), 0.0);
    }
    #[test]
    fn validation_rejects_bad_data_and_zero_duration_is_defined() {
        let mut p = line();
        assert!(p.validate().is_ok());
        p.tracks[0].keyframes[0].offset = f64::NAN;
        assert!(p.validate().is_err());
        let mut p = line();
        p.timing.duration = 0.0;
        assert_eq!(p.sample(0.0).get(Property::Tx), 100.0);
        assert_eq!(p.sample(-1.0).get(Property::Tx), 0.0);
        p.timing.iterations = Iterations::Infinite;
        assert!(p.validate().is_err());
        assert!(Curve::Steps { n: 0, start: false }.validate().is_err());
    }
}
